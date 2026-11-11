# Voice agent: local models for ASR, LLM and TTS (research, 11-11)

Scope: F16 and F17 of `design/voice-plan.md`, and R10 (each model is an
endpoint). Inputs: `docs/voice.md` sections 1, 2, 5, 7, 8, 9, 15, 17 to 19,
and `services/voice/README.md`. Today's reference stack is Gufo on an AMD GPU:
Qwen3-ASR-1.7B (realtime websocket), Qwen3-TTS-12Hz-1.7B-CustomVoice (Gufo
stream websocket), and a GGUF LLM on `/v1/chat/completions`. It takes about
4.8 s from the end of speech to the first sound. Sources are listed at the
end as [n]. Where a number comes from a third party and not from a model card
or paper, it says so. A number marked "est." is my estimate, to be measured.

## 1. Requirements as measurable criteria

The latency budget behind F9 (first sound under 1.5 s for a one-tool turn):
end wait 500 ms (`silence_ms` today: 800), ASR 250 ms, LLM round 1 (tool
call) 400 ms, tool 30 ms, LLM round 2 up to the first sentence 300 ms, TTS
first audio 150 ms. Today the playback buffer adds 1 s because the TTS is
slower than real time.

| # | Criterion | Target | How it is measured (section 4) |
|---|---|---|---|
| L1 | Tool-call accuracy on our tools | at least 95 % exact tool and key arguments; at least 98 % on confirm-gated tools (no wrong `apply_commands close`) | task set T, both languages |
| L2 | No call when none is needed; asks back when unclear | at least 95 % | task set T, class "chat" and "unclear" |
| L3 | German quality | replies in `reply_language`, formal "Sie", no English leaks; at least 4 of 5 in a blind rating; ASR WER at most 8 % on our German clips | judge plus a human spot check; ASR on the corpus |
| L4 | First token, warm cache | at most 400 ms per round with 22.7k tokens of cached prefix and up to 1.5k new tokens | `timings` of llama.cpp, or TTFT on the client side |
| L5 | Decode speed | at least 40 tok/s (a tool call of about 30 tokens then takes under 0.8 s); at least 15 tok/s on the CPU tier | client side |
| L6 | Context and cache | at least 32k context; the static prefix (prompt, docs, tools in a stable order) is reused across turns and rounds; a cold prefill of 22.7k tokens in at most 10 s on a GPU (41 s today) | `usage.prompt_tokens_details.cached_tokens`, or llama.cpp `tokens_cached` |
| A1 | ASR latency | final text at most 300 ms after the commit for 3 s of speech (today 235 to 545 ms) | `asr` stage of the turn log |
| A2 | ASR words | wake, sleep, cancel and yes/no words at least 98 % (today "Hallo Cora", "verbrechen"); bias by `asr_prompt` or hotwords | word matcher over the corpus |
| T1 | TTS first audio | at most 200 ms per sentence (today 196 to 490 ms) | `tts_first` |
| T2 | TTS speed | real-time factor at most 0.5, so `playback_buffer_ms` can drop to about 150 (today slower than real time) | PCM length over wall time |
| T3 | TTS German | intelligible: TTS then ASR round trip WER at most 5 %; German voice, not an accent | round trip plus listening |
| H | Memory for all three models at once | no GPU: at most 12 GB RAM; 8 GB GPU: at most 7.5 GB VRAM plus 8 GB RAM; 16 to 24 GB GPU: at most 90 % of VRAM | `nvidia-smi`, `rocm-smi`, RSS |
| Lic | Licence | weights and runtime OSI-like (Apache 2.0, MIT, BSD) or CC-BY; no non-commercial and no use-based restrictions in the reference stack; GPL only as a separate server process | model cards |

## 2. Candidates per stage

Runtimes with an OpenAI-compatible API:
- llama.cpp `llama-server`: `/v1/chat/completions` with tools (`--jinja`), prompt cache per slot, `--cache-reuse`; for hybrid models, also `--ctx-checkpoints` [16].
- vLLM: tool parsers (`hermes`, `qwen3_coder`, `mistral`), automatic prefix caching, `/v1/audio/transcriptions`, and a realtime path for Qwen3-ASR [5][6].
- Ollama: `/v1/chat/completions` with tools; less control over the cache and the template.
- whisper.cpp `whisper-server` and Speaches (faster-whisper, plus Kokoro or Piper; `/v1/audio/transcriptions`, `/v1/audio/speech`, `/v1/realtime`) [13][14].
- Piper HTTP server (`piper.http_server`): its own API, not OpenAI's [12].

### LLM

| Model | Size (active) | Licence | German | Tool calling (primary numbers) | Runtimes | Fit per tier |
|---|---|---|---|---|---|---|
| Qwen3.5-4B | 4B dense, hybrid DeltaNet | Apache 2.0 | 201 languages claimed; MMMLU, not per language [1] | BFCL-V4 50.3, TAU2 79.9 [1] | vLLM, SGLang, llama.cpp GGUF, Ollama | CPU: yes (Q4 about 2.7 GB est.); 8 GB: yes; fine-tune base |
| Qwen3.5-9B | 9B dense, hybrid | Apache 2.0 | as above; MMMLU 81.2 [1] | BFCL-V4 66.1, TAU2 79.1 [1] | as above | CPU: slow (Q4 about 6 GB est., 5 to 10 tok/s est.); 8 GB: best fit at Q4; 16 GB: Q8 |
| Qwen3.5-35B-A3B | 35B MoE (3B active) | Apache 2.0 | as above | BFCL-V4 67.3, TAU2 81.2 [2] | as above; MoE offload in llama.cpp | CPU with 32 GB RAM: decode works (3B active), prefill is slow; 24 GB: Q4 fits |
| Qwen3.5-27B | 27B dense | Apache 2.0 | as above | BFCL-V4 68.5, TAU2 79.0 [2] | as above | 24 GB at Q4, tight with the ASR and TTS |
| Gemma 4 E4B / 12B / 26B-A4B | 8B (4.5B eff.) / 12B / 25B (3.8B active) | Apache 2.0 | MMMLU 76.6 / 83.4 / 86.3 [3] | Tau2 42.2 / 69.0 / 68.2 [3]; native tools | vLLM, llama.cpp, Ollama | the second family to test; E4B and 12B also take audio (one model for ASR and LLM, later) |
| Ministral 3 3B / 8B / 14B (2512) | dense | Apache 2.0 [4] | European vendor; no German numbers found | function calling; vLLM `--tool-call-parser mistral` [4] | vLLM, llama.cpp, Ollama | the third family; plain attention, so the llama.cpp prompt cache is simple |

Notes:
- Qwen3.5 thinks by default. It needs `chat_template_kwargs: {enable_thinking: false}`; the `/no_think` switch of Qwen3 is gone [1]. `reasoning_effort: none` alone is not enough.
- Hybrid DeltaNet layers keep a recurrent state, not a KV cache per token. llama.cpp can then reuse only saved checkpoints of that state. Reports through 2026 show "forcing full prompt re-processing" on Qwen3.5, open bugs on restoring checkpoints, and fixes landing in HEAD [16]. This is the biggest risk for L4 and L6. Ministral and Gemma 4 use plain attention and do not have it. Measure the reuse before choosing.
- Third-party numbers that fit (not primary): a Reddit run puts Gemma 4 E4B at 65.3 % BFCL; Qwen3.5-9B leads on NexusRaven at 75 to 77 % [17]. Runtime parsers can drop parallel calls [17].
- The official BFCL leaderboard was last updated on 2026-04-12 [7]. Public benchmarks do not cover our schema or German speech, hence section 4.

### ASR

| Model | Size | Licence | German | Streaming / API | Runtimes | Fit |
|---|---|---|---|---|---|---|
| Qwen3-ASR-1.7B (today) / 0.6B | 1.7B / 0.6B | Apache 2.0 | yes. Averages over 12 languages on Fleurs: 4.90 vs Whisper-large-v3 5.27; no figure per language [5] | streaming with vLLM only; OpenAI transcription and chat endpoints [5]; the realtime buffer in vLLM [6] | vLLM, Gufo; a GGUF of 0.6B for transcribe.cpp | GPU tiers; 0.6B for 8 GB |
| Parakeet-TDT-0.6B-v3 | 0.6B | CC-BY-4.0 | German WER: FLEURS 5.04, CoVoST 4.84 [8] | batch, word timestamps, automatic language | NeMo, community OpenAI-compatible FastAPI, a GGUF runtime [8][15] | CPU and 8 GB: the best mix of accuracy and size; no prompt biasing (A2 needs testing) |
| Voxtral-Mini-4B-Realtime-2602 | 4B | Apache 2.0 | one of 13 languages | native streaming; at a delay of 480 ms on par with Whisper [9] | vLLM | 16 to 24 GB; true streaming could remove the commit wait |
| Whisper large-v3-turbo | 0.8B (4 decoder layers) | MIT | good; Fleurs 8.16 vs 12.60 for Qwen3-ASR on 10 more languages [5] | batch; initial prompt biases words | whisper.cpp server, faster-whisper or Speaches [13][14] | CPU: about 0.5 x real time on M2 (other runtime) [13]; the fallback |

### TTS

| Model | Size | Licence | German | API | Known numbers | Fit |
|---|---|---|---|---|---|---|
| Qwen3-TTS-12Hz 1.7B (today) / 0.6B | 1.7B / 0.6B | Apache 2.0 | yes, 10 languages [10] | Gufo websocket; vLLM-Omni offline only [10] | claims 97 ms to the first packet [10]; measured here: slower than real time on 1.7B | GPU; try 0.6B for T2 |
| Piper (piper1-gpl), `de_DE-thorsten-*` | 15 to 60 M | engine GPL-3.0; voices each have their own licence (Thorsten: CC0) [12] | yes, native voices | own HTTP server; Speaches `/v1/audio/speech` [14] | about 1.5 s first audio in Picovoice's CPU test, vendor setup [11] | CPU tier; GPL is fine as a separate process |
| Kokoro-82M | 82 M | Apache 2.0 | no German voice [18] | Speaches, Kokoro-FastAPI `/v1/audio/speech` | about 2.9 s first audio in Picovoice's CPU test [11] | English only; not the reference |
| Chatterbox Multilingual | 0.5B | MIT | yes, 23 languages; zero-shot clone; watermark [19] | community OpenAI wrappers | claims under 200 ms; Picovoice measured the Turbo variant at 44.7 s on CPU [11][19] | 16 to 24 GB GPU only |

## 3. Recommendation: a reference stack per tier

All three tiers use the same agent and the same settings keys; only URLs and
models change (section 6). The prompt must shrink on the small tiers: 22.7k
tokens of prompt and docs is the main cost on a CPU and in a cold start. Use
`context_docs` with a "slim" profile of about 4k tokens (prompt, tool
schemas, two short sections of `docs/api.md`).

| Tier | LLM | ASR | TTS | Why |
|---|---|---|---|---|
| No GPU (8 cores, 32 GB RAM) | Qwen3.5-4B Q4_K_M on llama.cpp, slim prompt; option Ministral 3 3B if the hybrid cache does not hold | Parakeet-TDT-0.6B-v3 (int8, CPU); whisper.cpp large-v3-turbo q5 as fallback | Piper `de_DE-thorsten-high` and an English Piper voice | prefill of 22.7k tokens on a CPU takes minutes, so the slim prompt is mandatory; 4B decodes at 15 to 25 tok/s est.; Piper is the only German TTS that is fast on a CPU. Expect first sound around 2 to 3 s est. |
| 8 GB GPU | Qwen3.5-9B Q4_K_M on llama.cpp, all layers on the GPU, 32k context (about 6.5 GB est.) | Parakeet v3 on the CPU (or Qwen3-ASR-0.6B if VRAM is left) | Piper on the CPU | the 9B has the best primary tool numbers that fit (BFCL-V4 66.1) [1]; ASR and TTS on the CPU keep the VRAM for the LLM |
| 16 to 24 GB GPU (today's Gufo class) | 24 GB: Qwen3.5-35B-A3B Q4 (MoE: speed of 3B, quality near 27B) [2]; 16 GB: Qwen3.5-9B Q6 to Q8 | Qwen3-ASR-1.7B (today); test Voxtral Realtime for streaming | Qwen3-TTS-12Hz-0.6B; Piper as a fast fallback | keeps the working stack; 0.6B TTS targets T2, so the 1 s playback buffer can go |

Before this is final, run section 4 on Qwen3.5, Gemma 4 and Ministral 3. L4
and L6 decide between the hybrid Qwen3.5 and plain attention (Q-V3 fixes the machine).

## 4. Benchmark harness

One binary path, so the harness measures what the agent does:
`chora-voice eval llm|asr|tts|e2e --set FILE` uses the same `llm.rs`,
`asr`, `tts` clients and the same settings file, so any endpoint is a
change of `*_url` and `*_model`. The results go to JSONL with one line per
case; `eval report` makes a table per model.

Task set T (LLM, about 300 cases, YAML in `services/voice/eval/`):
- Tools come from a frozen MCP fixture: the `tools/list` of a headless
  session with `chora-tiling`, `chora-mc` and `chora-assistant` (the 25 tools
  of WP5), and replies recorded for `query_scene`, `tiling.get_layout`,
  `describe_screen` (text only), `agent_list` and `settings.get`. No real
  calls, so runs are deterministic and safe.
- Classes, each in German and English, from real ASR text (with its errors,
  such as "Hallo Cora"):
  1. look-ups: windows, layout, plugins, helpers, settings (one round, then an answer);
  2. acts: focus, move, close windows, the cube, views, `run_program`,
     `configure_output`, `call_method`; the expected call and the confirm rule
     it hits (section 5);
  3. two-step chains: `query_scene`, then `apply_commands` on the right id;
  4. helpers and chat: `agent_start`, `agent_say`, `chat.show` (sections 18, 19);
  5. no tool: small talk, a question about chora that the docs answer;
  6. unclear or unsafe: ask back; never `inject_input` for a key-only question (R4, R7);
  7. after a declined confirm or a cancel note: no retry of the same call;
  8. long history: a 20-turn session near `history_tokens`.
- Scoring per case: the tool name exact; key arguments by a matcher per tool
  (ids, names, command kinds; free text by contains); the count of rounds; the
  reply language and the formal "Sie" (a rule, plus an LLM judge on a
  10 % sample, checked by hand); a hard fail for any call that a confirm rule
  would gate and that the case does not expect.
- Metrics: L1, L2 per class and language; TTFT per round, prefill tok/s,
  decode tok/s, cached tokens; total time to the first sentence; peak
  VRAM and RAM. Each case runs three times, warm and cold.

ASR set: the replay corpus of section 12 (`hallo-chora`, `turns-noise`,
`echo-barge`, `echo-loud`) plus about 200 new clips: 100 German, 100
English, from at least 4 speakers, on the reference mic with the echo
canceller, also over agent speech. `labels.txt` gains a `text` line with the
reference. Metrics: WER and CER, accuracy of the control words through the
word matcher (A2), latency after the commit, false text on noise.

TTS set: 100 reply sentences from T, German and English, also numbers,
"z. B.", window and plugin names. Metrics: first audio, RTF, round trip
WER through the reference ASR, and a blind A/B rating by the user.

End to end: `chora-voice replay --live` feeds the corpus in real time
into the full agent against the endpoints and reads the turn summary of
section 11 (`first_sound`, outcome). This is the number that F9 tracks.

## 5. Fine-tuning study (F17)

Will a 1 to 8 B model with LoRA win? The evidence:
- TinyAgent: 1.1B and 7B models, tuned on about 80k synthetic calls for 16
  macOS functions, matched or beat GPT-4-Turbo on that task [20]. NVIDIA's
  position paper argues the same for narrow agent calls [21]. Our tools are
  a narrow set too.
- Out of the box, Qwen3.5-4B is already at TAU2 79.9 [1]. The gap to close
  on our tools may be small. The likely gain is not accuracy alone but a
  shorter prompt: the docs (most of the 22.7k tokens) go into the weights,
  and that cuts the prefill on the CPU and 8 GB tiers.
- Our tools change: plugins add methods, and the agent reloads tools each
  turn. A model tuned on one schema can break on the next one. So the
  schemas stay in the prompt, and training randomises their set and order.

Data:
- Synthetic: 5k to 10k dialogues, generated with an open teacher
  (Qwen3.5-397B-A17B or 122B-A10B, Apache 2.0) over the fixture of section 4,
  German and English half each. They include confirm declines, cancel notes,
  ASR noise (word swaps from the corpus) and the no-tool and ask-back classes.
  Filter: the call must validate against the schema and pass the scorer of
  section 4.
- Recorded: 300 to 500 real utterances from `record_dir` sessions, with
  consent, as ASR text; labelled by hand. They are only for the test split,
  never for training, so the score shows real speech.
- Splits by template and by tool, so the test set has phrasings and tool
  combinations that training never saw.

Method: bf16 LoRA (rank 16 to 32, all linear layers), loss on assistant turns
only, 2 to 3 epochs, Unsloth or TRL. Unsloth advises bf16 LoRA over QLoRA for
Qwen3.5 and gives VRAM needs of 10 GB for 4B and 22 GB for 9B [22]. Then
merge, convert to GGUF Q4/Q8, and serve with the same llama-server.

Hardware and cost (est.): a 4B fits a local 16 GB GPU; a 9B needs 24 GB or a
rented 80 GB card for about 3 to 6 h per run, roughly 10 to 40 USD a run at
public cloud rates. Data generation with a local teacher costs GPU time
only; with a hosted open model, about 10M tokens. Allow 3 to 5 runs.

Risks:
- Overfitting to the templates: a high score on synthetic data, failure on real speech. Hence the recorded test split.
- Loss of general skill (German small talk, docs questions): keep 20 % general chat data and check MMMLU-de.
- Schema drift, as above.
- Licences: the bases are Apache 2.0, so a tuned model may be shipped. Teacher outputs from a closed API may be barred by its terms for training competing models, so use an open teacher. Recordings fall under R9 (no telemetry; only with `record_dir` and consent).
- Upkeep: each new base model or tool family means a new run.

Go/no-go: no-go now. Go only when all of these hold:
1. Section 4 runs on the reference hardware.
2. The best general model of the tier, with the slim prompt, misses L1 (95 %) or L4 (400 ms).
3. A pilot (4B, 2k synthetic dialogues, one run) beats that model by at least 5 points on the recorded test split.
4. The pilot loses at most 2 points on the general checks.
Ship a tuned model only if it keeps winning on a new tool schema (one
plugin added that it never saw).

## 6. Interchangeability: config keys

Already in section 9 (D62): `asr_url`, `tts_url`, `llm_url` with `*_model`,
`*_api_key_env` and `*_api_key_file`, `*_headers` and TLS; `asr_language`,
`asr_prompt`, `voice`, `tts_language`, `reply_language`; `context_tokens`
(else `/v1/models`), `reasoning_effort`; `system_prompt`, `context_docs`,
`prompt_file`; timeouts per stage. So the LLM is already just an endpoint for
any OpenAI chat server with tools. The ASR and TTS are endpoints only for
the two websocket dialects of Gufo (OpenAI realtime transcription, and the
`audio.start` stream of section 8).

Missing for R10:

| Key | Values | Why |
|---|---|---|
| `asr_api` | `realtime` (today), `transcriptions` | POST `/v1/audio/transcriptions` with a WAV per utterance: whisper.cpp, Speaches, Parakeet servers, vLLM |
| `asr_rate` | `24000` (today), `16000` | Whisper and Parakeet take 16 kHz; the VAD path already has the 3:2 low-pass |
| `tts_api` | `stream` (today), `speech` | POST `/v1/audio/speech` with `stream: true`, `response_format: pcm`: Speaches, Kokoro-FastAPI, OpenAI, vLLM-Omni later |
| `tts_rate` | `24000` | Piper voices are often 22050 Hz; the agent resamples for `playback_cmd` |
| `voices` | `{de: thorsten, en: ...}` | Piper and Kokoro voices belong to one language; today `voice` is one name |
| `llm_extra_body` | a map, merged into each request | `chat_template_kwargs.enable_thinking: false` (Qwen3.5), `cache_prompt`, `id_slot`, `top_k` |
| `llm_temperature`, `llm_max_tokens`, `llm_parallel_tool_calls` | numbers, bool | sampling for tool calls (low temperature); some servers drop parallel calls [17] |
| `prompt_profile` | `full`, `slim` | a named set of `context_docs` for small tiers (section 3) |

The turn log should show `cached_tokens` per round (L6). A router (a small
model for tool rounds) would need `llm_tool_url`; it waits for a need in section 4.

## Sources

1. Qwen3.5-9B model card: https://huggingface.co/Qwen/Qwen3.5-9B
2. Qwen3.5-35B-A3B model card: https://huggingface.co/Qwen/Qwen3.5-35B-A3B
3. Gemma 4 model card: https://ai.google.dev/gemma/docs/core/model_card_4
4. Ministral 3 8B model card: https://docs.mistral.ai/models/model-cards/ministral-3-8b-25-12 and https://www.lmstudio.ai/models/ministral
5. Qwen3-ASR repository and benchmarks: https://github.com/QwenLM/Qwen3-ASR
6. vLLM `qwen3_asr_realtime`: https://docs.vllm.ai/en/v0.32.0/api/vllm/model_executor/models/qwen3_asr_realtime/
7. Berkeley Function Calling Leaderboard: https://gorilla.cs.berkeley.edu/leaderboard.html
8. Parakeet-TDT-0.6B-v3 model card: https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3
9. Voxtral Realtime paper: https://arxiv.org/abs/2602.11298 and https://mistral.ai/news/voxtral-transcribe-2
10. Qwen3-TTS repository: https://github.com/QwenLM/Qwen3-TTS
11. Picovoice on-device TTS benchmark (vendor): https://picovoice.ai/blog/on-device-tts/
12. Piper (OHF-Voice/piper1-gpl): https://github.com/OHF-Voice/piper1-gpl and the Thorsten voice: https://huggingface.co/cstr/piper-voices-GGUF
13. whisper.cpp server: https://github.com/ggml-org/whisper.cpp, and the turbo timing on CPU (ailia): https://medium.com/axinc-ai/whisper-large-v3-turbo-high-accuracy-and-fast-speech-recognition-model-be2f6af77bdc
14. Speaches: https://speaches-ai.github.io/speaches
15. Parakeet OpenAI-compatible server (community): https://github.com/groxaxo/parakeet-tdt-0.6b-v3-fastapi-openai
16. llama.cpp hybrid cache issues: https://github.com/ggml-org/llama.cpp/issues/22384, https://github.com/ggml-org/llama.cpp/issues/21383, https://github.com/ggml-org/llama.cpp/pull/19493
17. Third-party tool-call tests (not primary): https://reddit.sentinel-team.org/posts/1sb84oy/snapshots/2026-04-03T21%3A20%3A24.375333Z and https://d-central.tech/local-llm-agent-capability/
18. Kokoro voices: https://huggingface.co/hexgrad/Kokoro-82M/blob/main/VOICES.md
19. Chatterbox model card: https://huggingface.co/ResembleAI/chatterbox
20. TinyAgent: Function Calling at the Edge: https://arxiv.org/abs/2409.00608
21. Small Language Models are the Future of Agentic AI: https://arxiv.org/abs/2506.02153
22. Unsloth, Qwen3.5 fine-tuning: https://unsloth.ai/docs/models/qwen3.5.md
