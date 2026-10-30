#!/bin/sh
# A fake ACP agent for the tests of chora-voice (docs/voice.md section
# 18): it answers by the order of the requests, not by their ids. Each
# prompt gets a plan, a tool call, a question and a long answer; the
# answer says whether the question got a yes. FAKE_ACP_DELAY (seconds)
# slows each step down, for screenshots.
delay=${FAKE_ACP_DELAY:-0}
step() { sleep "$delay"; }
say() {
    printf '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"s1","update":%s}}\n' "$1"
}
read -r _ || exit 0
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1,"agentCapabilities":{}}}'
read -r _ || exit 0
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"s1"}}'
id=3
while read -r prompt; do
    case "$prompt" in *session/prompt*) ;; *) continue ;; esac
    step
    say '{"sessionUpdate":"plan","entries":[{"content":"Code lesen","priority":"high","status":"completed"},{"content":"Datei schreiben","priority":"high","status":"in_progress"}]}'
    say '{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"Ich lese den Code."}}'
    say '{"sessionUpdate":"tool_call","toolCallId":"t1","title":"Read src/main.rs","kind":"read","status":"pending"}'
    step
    echo '{"jsonrpc":"2.0","id":"p1","method":"session/request_permission","params":{"sessionId":"s1","toolCall":{"toolCallId":"t2","title":"Write notes.txt","kind":"edit"},"options":[{"optionId":"yes","name":"Allow","kind":"allow_once"},{"optionId":"no","name":"Reject","kind":"reject_once"}]}}'
    read -r answer || exit 0
    case "$answer" in
    *'"yes"'*) word="Erlaubt" ;;
    *) word="Abgelehnt" ;;
    esac
    step
    say '{"sessionUpdate":"plan","entries":[{"content":"Code lesen","priority":"high","status":"completed"},{"content":"Datei schreiben","priority":"high","status":"completed"}]}'
    say "{\"sessionUpdate\":\"agent_message_chunk\",\"content\":{\"type\":\"text\",\"text\":\"$word: die Datei ist fertig. \"}}"
    say '{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"Danach habe ich noch alle Tests laufen lassen und die Ergebnisse in einer langen Liste mit vielen Einzelheiten zusammengefasst, die niemand vorgelesen haben will."}}'
    echo "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"stopReason\":\"end_turn\"}}"
    id=$((id + 1))
done
