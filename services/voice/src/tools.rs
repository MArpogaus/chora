//! The tools of the LLM: `choractl mcp` through an rmcp client, with
//! reconnect and a timeout per call, and the rules of the spoken
//! confirmation (docs/voice.md sections 5 and 6).

use std::{
    collections::HashMap,
    path::Path,
    pin::Pin,
    time::{Duration, Instant},
};

use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, CallToolResponse, ContentBlock},
    service::{RoleClient, RunningService},
    transport::TokioChildProcess,
};
use serde_json::{Value, json};

use crate::{agents::Agents, coder::Coder, config::Confirm};

pub type Service = RunningService<RoleClient, ()>;
type Connecting = Pin<Box<dyn Future<Output = Result<Service, String>> + Send>>;
/// Opens a new MCP session: `choractl mcp` as a child, or a fake.
pub type Connect = Box<dyn Fn() -> Connecting + Send + Sync>;

/// The longest wait between two connection attempts.
const MAX_BACKOFF: Duration = Duration::from_secs(5);

pub struct Tools {
    connect: Connect,
    service: Option<Service>,
    /// The function specs for the LLM.
    pub specs: Vec<Value>,
    /// The function name of the LLM and the MCP name: function names
    /// take no dots, plugin methods have one.
    names: HashMap<String, String>,
    pub skipped: Vec<String>,
    pub timeout: Duration,
    /// The LLM reads the pictures of the results.
    pub vision: bool,
    backoff: Duration,
    retry_at: Instant,
    /// The coding tools, when `coder_cmd` has a harness.
    pub coder: Option<Coder>,
    /// The tools of the ACP agents, when `agent_cmd` has a command.
    pub agents: Option<Agents>,
}

/// `CMD --name NAME mcp` as a child process; its stderr goes to ours.
pub fn child(cmd: Vec<String>, name: String) -> Connect {
    Box::new(move || {
        let (cmd, name) = (cmd.clone(), name.clone());
        Box::pin(async move {
            let (program, args) = cmd.split_first().ok_or("chora_mcp is empty")?;
            let mut command = tokio::process::Command::new(program);
            command.args(args).args(["--name", &name, "mcp"]);
            let transport =
                TokioChildProcess::new(command).map_err(|e| format!("{program}: {e}"))?;
            ().serve(transport)
                .await
                .map_err(|e| format!("{program}: {e}"))
        })
    })
}

impl Tools {
    /// The tools of `choractl mcp` under the client name `name`.
    pub fn chora(config: &crate::config::Config, name: &str) -> Self {
        Self::new(
            child(config.chora_mcp.clone(), name.to_owned()),
            config.skipped_tools.clone(),
            Duration::from_secs(config.tool_timeout_s.into()),
        )
    }

    pub fn new(connect: Connect, skipped: Vec<String>, timeout: Duration) -> Self {
        Self {
            connect,
            service: None,
            specs: Vec::new(),
            names: HashMap::new(),
            skipped,
            timeout,
            vision: false,
            backoff: Duration::ZERO,
            retry_at: Instant::now(),
            coder: None,
            agents: None,
        }
    }

    fn connected(&self) -> Option<&Service> {
        self.service.as_ref().filter(|s| !s.is_transport_closed())
    }

    /// Connects again after the child ended, at most once per backoff
    /// (0.5 s to 5 s), and loads the tools. The tools of the last session
    /// stay while chora is away. The coding tools come last.
    pub async fn refresh(&mut self) {
        self.load().await;
        let mut own = self.coder.as_ref().map(Coder::specs).unwrap_or_default();
        own.extend(self.agents.as_ref().map(Agents::specs).unwrap_or_default());
        self.specs.retain(|s| {
            let name = s["function"]["name"].as_str().unwrap_or_default();
            !crate::coder::TOOLS.contains(&name) && !crate::agents::TOOLS.contains(&name)
        });
        for spec in own {
            let name = spec["function"]["name"]
                .as_str()
                .unwrap_or_default()
                .to_owned();
            self.names.insert(name.clone(), name);
            self.specs.push(spec);
        }
    }

    async fn load(&mut self) {
        if self.connected().is_none() {
            if Instant::now() < self.retry_at {
                return;
            }
            match (self.connect)().await {
                Ok(service) => {
                    crate::health::ok("mcp");
                    self.service = Some(service);
                    self.backoff = Duration::ZERO;
                }
                Err(e) => {
                    crate::health::down("mcp", &e);
                    self.service = None;
                    self.backoff =
                        (self.backoff * 2).clamp(Duration::from_millis(500), MAX_BACKOFF);
                    self.retry_at = Instant::now() + self.backoff;
                    crate::log::warn(&format!("mcp: {e}; again in {:?}", self.backoff));
                    return;
                }
            }
        }
        let Some(service) = self.connected() else {
            return;
        };
        match service.list_all_tools().await {
            Ok(tools) => {
                let tools: Vec<_> = tools
                    .into_iter()
                    .filter(|t| !self.skipped.iter().any(|s| glob(s, &t.name)))
                    .collect();
                self.names.clear();
                self.specs = tools
                    .iter()
                    .map(|tool| {
                        let name = tool.name.replace('.', "__");
                        self.names.insert(name.clone(), tool.name.to_string());
                        json!({"type": "function", "function": {
                            "name": name,
                            "description": tool.description,
                            "parameters": tool.input_schema,
                        }})
                    })
                    .collect();
            }
            Err(e) => crate::log::warn(&format!("mcp tools/list: {e}")),
        }
    }

    /// The MCP name of a function of the LLM.
    pub fn mcp_name<'a>(&'a self, function: &'a str) -> Option<&'a str> {
        self.names.get(function).map(String::as_str)
    }

    /// Runs a tool and answers the content of the tool message: the text,
    /// and the pictures when the LLM reads them. An error is a text that
    /// starts with `error:`, which the LLM reads.
    pub async fn call(&mut self, function: &str, args: &Value) -> Value {
        let Some(name) = self.mcp_name(function).map(str::to_owned) else {
            return format!("error: no tool {function}").into();
        };
        let Value::Object(args) = args.clone() else {
            return "error: the arguments are no JSON object".into();
        };
        if let Some(coder) = &self.coder
            && crate::coder::TOOLS.contains(&name.as_str())
        {
            return coder.call(&name, &Value::Object(args));
        }
        if let Some(agents) = &self.agents
            && crate::agents::TOOLS.contains(&name.as_str())
        {
            return agents.call(&name, &Value::Object(args));
        }
        self.refresh().await;
        let Some(service) = self.connected() else {
            return "error: chora is not connected".into();
        };
        let params = CallToolRequestParams::new(name).with_arguments(args);
        // A dropped call sends notifications/cancelled.
        match tokio::time::timeout(self.timeout, service.call_tool_once(params)).await {
            Err(_) => format!(
                "error: no answer within {:?}; the call was cancelled, its result is unknown",
                self.timeout
            )
            .into(),
            Ok(Err(_)) if service.is_transport_closed() => {
                crate::health::down("mcp", "the connection was lost");
                "error: the connection to chora was lost; the result is unknown".into()
            }
            Ok(Err(e)) => format!("error: {e}").into(),
            Ok(Ok(CallToolResponse::Complete(result))) => {
                content(&result.content, result.is_error == Some(true), self.vision)
            }
            Ok(Ok(_)) => "error: the tool asked for input, which the agent cannot give".into(),
        }
    }
}

/// The text of the blocks (or `done`), and the pictures as `image_url`
/// parts when the LLM reads them.
fn content(blocks: &[ContentBlock], error: bool, vision: bool) -> Value {
    let texts: Vec<&str> = blocks
        .iter()
        .filter_map(|b| b.as_text().map(|t| t.text.as_str()))
        .collect();
    let mut text = if texts.is_empty() {
        "done".to_owned()
    } else {
        texts.join("\n")
    };
    if error {
        text.insert_str(0, "error: ");
    }
    let images: Vec<Value> = blocks
        .iter()
        .filter_map(ContentBlock::as_image)
        .filter(|_| vision)
        .map(|i| {
            json!({"type": "image_url",
                "image_url": {"url": format!("data:{};base64,{}", i.mime_type, i.data)}})
        })
        .collect();
    if images.is_empty() {
        return text.into();
    }
    let mut parts = vec![json!({"type": "text", "text": text})];
    parts.extend(images);
    parts.into()
}

/// `*` matches any text, also an empty one.
fn glob(pattern: &str, text: &str) -> bool {
    let mut parts = pattern.split('*');
    let first = parts.next().unwrap_or_default();
    let Some(mut rest) = text.strip_prefix(first) else {
        return false;
    };
    let parts: Vec<&str> = parts.collect();
    let Some((last, middle)) = parts.split_last() else {
        return rest.is_empty();
    };
    for part in middle {
        match rest.find(part) {
            Some(i) => rest = &rest[i + part.len()..],
            None => return false,
        }
    }
    rest.ends_with(last)
}

/// The first rule that asks before the call of `tool` (its MCP name)
/// with `args`.
pub fn rule<'a>(rules: &'a [Confirm], tool: &str, args: &Value) -> Option<&'a Confirm> {
    rules.iter().find(|r| {
        let kinds = || {
            args["commands"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_object)
                .flat_map(|c| c.keys())
        };
        let program = program(args);
        glob(&r.tool, tool)
            && (r.commands.is_empty() || kinds().any(|k| r.commands.contains(k)))
            && (r.argv0.is_empty() || program.is_some_and(|p| r.argv0.iter().any(|a| a == p)))
            && r.method.as_ref().is_none_or(|m| {
                args["method"]
                    .as_str()
                    .is_some_and(|method| glob(m, method))
            })
    })
}

/// The file name of the program of `run_program`.
pub fn program(args: &Value) -> Option<&str> {
    args["argv"][0].as_str().map(|p| {
        Path::new(p)
            .file_name()
            .map_or(p, |n| n.to_str().unwrap_or(p))
    })
}

#[cfg(test)]
pub mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use rmcp::{
        ErrorData, ServerHandler,
        model::{
            CallToolResult, ListToolsResult, PaginatedRequestParams, ServerCapabilities,
            ServerConfig, Tool,
        },
        service::{RequestContext, RoleServer},
    };

    use super::*;
    use crate::config::Config;

    /// A fake `choractl mcp`: `echo` answers its arguments, `sleep`
    /// never answers, `picture` answers a PNG, `quit` ends the session,
    /// `tiling.set_layout` has a dot.
    #[derive(Clone)]
    pub struct Fake {
        pub calls: Arc<AtomicUsize>,
        quit: Arc<tokio::sync::Notify>,
    }

    impl ServerHandler for Fake {
        fn get_info(&self) -> ServerConfig {
            ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
        }

        fn list_tools(
            &self,
            _page: Option<PaginatedRequestParams>,
            _context: RequestContext<RoleServer>,
        ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + rmcp::service::MaybeSendFuture + '_
        {
            let schema = json!({"type": "object"}).as_object().unwrap().clone();
            let tools = [
                "echo",
                "sleep",
                "picture",
                "quit",
                "tiling.set_layout",
                "capture",
            ]
            .map(|name| Tool::new(name, format!("the tool {name}"), schema.clone()));
            std::future::ready(Ok(ListToolsResult::with_all_items(tools.to_vec())))
        }

        async fn call_tool(
            &self,
            request: CallToolRequestParams,
            context: RequestContext<RoleServer>,
        ) -> Result<CallToolResponse, ErrorData> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let args = Value::Object(request.arguments.unwrap_or_default());
            let blocks = match request.name.as_ref() {
                "sleep" => {
                    context.ct.cancelled().await;
                    vec![]
                }
                "quit" => {
                    self.quit.notify_one();
                    std::future::pending().await
                }
                "picture" => vec![
                    ContentBlock::image("UE5H", "image/png"),
                    ContentBlock::text("a screen"),
                ],
                _ => vec![ContentBlock::text(args.to_string())],
            };
            Ok(CallToolResult::success(blocks).into())
        }
    }

    async fn serve(fake: Fake, io: tokio::io::DuplexStream) {
        if let Ok(s) = fake.serve(io).await {
            let _ = s.waiting().await;
        }
    }

    /// Tools on a fake server in this process; each connect is a new
    /// session, the first `fail` connects fail.
    pub fn fake(fail: usize) -> (Tools, Arc<AtomicUsize>) {
        let calls = Arc::<AtomicUsize>::default();
        let fake = Fake {
            calls: calls.clone(),
            quit: Arc::default(),
        };
        let tries = Arc::new(AtomicUsize::new(0));
        let connect: Connect = Box::new(move || {
            let (fake, tries) = (fake.clone(), tries.clone());
            Box::pin(async move {
                if tries.fetch_add(1, Ordering::SeqCst) < fail {
                    return Err("no choractl".into());
                }
                // A relay between the two ends: `quit` cuts it, as a
                // child that ends.
                let (client, mut near) = tokio::io::duplex(1 << 16);
                let (mut far, server) = tokio::io::duplex(1 << 16);
                let relay = tokio::spawn(async move {
                    let _ = tokio::io::copy_bidirectional(&mut near, &mut far).await;
                });
                let quit = fake.quit.clone();
                tokio::spawn(async move {
                    quit.notified().await;
                    relay.abort();
                });
                tokio::spawn(serve(fake, server));
                ().serve(client).await.map_err(|e| e.to_string())
            })
        });
        let config = Config::default();
        let tools = Tools::new(connect, config.skipped_tools, Duration::from_millis(300));
        (tools, calls)
    }

    #[tokio::test]
    async fn tools_load_without_the_skipped_and_dots_become_two_underscores() {
        let (mut tools, _) = fake(0);
        tools.refresh().await;
        let names: Vec<_> = tools
            .specs
            .iter()
            .map(|s| s["function"]["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            ["echo", "sleep", "picture", "quit", "tiling__set_layout"]
        );
        assert_eq!(
            tools.mcp_name("tiling__set_layout"),
            Some("tiling.set_layout")
        );
        let answer = tools
            .call("tiling__set_layout", &json!({"layout": "stack"}))
            .await;
        assert_eq!(answer, r#"{"layout":"stack"}"#);
        assert_eq!(tools.call("nope", &json!({})).await, "error: no tool nope");
    }

    #[tokio::test]
    async fn the_coding_tools_come_with_a_harness_and_ask_by_their_rule() {
        let (mut tools, _) = fake(0);
        let (tx, _rx) = std::sync::mpsc::channel();
        let coder = Coder::new(
            Vec::new(),
            std::env::temp_dir().join("chora-voice-none"),
            tx,
        );
        tools.coder = Some(coder.clone());
        tools.refresh().await;
        assert_eq!(tools.specs.len(), 5, "no harness, no coding tools");
        coder.set_cmd(vec!["true".into()]);
        tools.refresh().await;
        tools.refresh().await;
        let names: Vec<_> = tools.specs[5..]
            .iter()
            .map(|s| s["function"]["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, crate::coder::TOOLS, "once each, after the others");
        let status = tools.call("code_status", &json!({})).await;
        assert_eq!(status, "no coding task runs");
        let rules = Config::default().confirm;
        let name = tools.mcp_name("code_task").unwrap();
        assert!(rule(&rules, name, &json!({"task": "x"})).is_some());
    }

    #[tokio::test]
    async fn a_slow_call_times_out_and_pictures_go_only_to_an_llm_that_sees() {
        let (mut tools, _) = fake(0);
        tools.refresh().await;
        let start = Instant::now();
        let answer = tools.call("sleep", &json!({})).await;
        assert!(answer.as_str().unwrap().contains("no answer"), "{answer}");
        assert!(start.elapsed() < Duration::from_secs(2));
        assert_eq!(tools.call("picture", &json!({})).await, "a screen");
        tools.vision = true;
        let answer = tools.call("picture", &json!({})).await;
        assert_eq!(
            answer[1]["image_url"]["url"], "data:image/png;base64,UE5H",
            "{answer}"
        );
    }

    #[tokio::test]
    async fn a_failed_connect_waits_its_backoff_then_connects() {
        let (mut tools, _) = fake(1);
        tools.refresh().await;
        assert_eq!(tools.specs, Vec::<Value>::new());
        let answer = tools.call("echo", &json!({})).await;
        assert_eq!(answer, "error: no tool echo");
        tokio::time::sleep(Duration::from_millis(600)).await;
        tools.refresh().await;
        assert_eq!(tools.call("echo", &json!({"a": 1})).await, r#"{"a":1}"#);
    }

    #[tokio::test]
    async fn a_lost_connection_answers_unknown_and_connects_again() {
        let (mut tools, calls) = fake(0);
        tools.refresh().await;
        let answer = tools.call("quit", &json!({})).await;
        assert!(answer.as_str().unwrap().contains("lost"), "{answer}");
        assert_eq!(tools.call("echo", &json!({"b": 2})).await, r#"{"b":2}"#);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn rules_match_by_tool_command_program_and_method() {
        let rules = Config::default().confirm;
        let prompt = |tool: &str, args: Value| rule(&rules, tool, &args).map(|r| r.prompt.as_str());
        let remove = json!({"commands": [{"create": {}}, {"remove": {"id": "x"}}]});
        assert_eq!(
            prompt("apply_commands", remove),
            Some("Shall I remove that?")
        );
        assert_eq!(
            prompt("apply_commands", json!({"commands": [{"create": {}}]})),
            None
        );
        for argv in [json!(["/usr/bin/systemctl", "poweroff"]), json!(["foot"])] {
            assert_eq!(
                prompt("run_program", json!({ "argv": argv })),
                Some("Shall I start the program {program}?")
            );
        }
        assert_eq!(
            prompt("tiling.restart", json!({})),
            Some("Shall I restart the plugin?")
        );
        assert!(
            prompt(
                "call_method",
                json!({"client": "xp", "method": "xp.restart"})
            )
            .is_some()
        );
        assert_eq!(prompt("call_method", json!({"method": "xp.layout"})), None);
        assert_eq!(prompt("query_scene", json!({})), None);
    }

    #[test]
    fn glob_takes_stars_anywhere() {
        for (pattern, text, matches) in [
            ("*", "", true),
            ("a*c", "abc", true),
            ("a*c", "abd", false),
            ("*.restart", "tiling.restart", true),
            ("*.restart", "restart", false),
            ("a*b*c", "aXbYc", true),
            ("ab", "abc", false),
        ] {
            assert_eq!(glob(pattern, text), matches, "{pattern} {text}");
        }
    }
}
