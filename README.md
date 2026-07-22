# Panca
Server benchmarking tool. Still a WIP.

```bash
Server benchmarking tool

Usage: panca [OPTIONS] <URL>

Arguments:
  <URL>  URL of the endpoint to benchmark

Options:
  -t, --threads <THREADS>
          Number of worker threads (one more "display thread" is spawned for metrics) [default: 1]
  -c, --concurrent-requests <CONCURRENT_REQUESTS>
          Number of concurrent requests per thread [default: 10]
  -r, --requests <REQUESTS>
          Total number of requests sent by each thread. If not specified, requests are sent indefinitely
  -m, --method <METHOD>
          HTTP method [default: get] [possible values: get, post]
      --json <JSON>
          JSON payload to send as the request body. Sets 'Content-Type: application/json' header
      --metrics-interval-seconds <METRICS_INTERVAL_SECONDS>
          How often metric are updated (in seconds) [default: 1]
      --display-interval-seconds <DISPLAY_INTERVAL_SECONDS>
          How often metrics are displayed to stdout (in seconds) [default: 1]
      --plugins <PLUGINS>
          Metrics plugins (e.g., basic,llamacpp) [default: basic]
  -h, --help
          Print help
  -V, --version
          Print version
```

## Examples
### basic bench
```bash
cargo run --release -- http://127.0.0.1:54860/v1/models -t 5 -c 2
```
Output:
```bash
----Snapshot 5----
[basic][thread=0] Mean Req/s: 19280.8436
[basic][thread=0] Mean Response time (ms): 0.0979
[basic][thread=0] Total requests sent: 77126
[basic][thread=0] Errors: 0% (0/77126)
[basic][thread=1] Mean Req/s: 19278.0248
[basic][thread=1] Mean Response time (ms): 0.0978
[basic][thread=1] Total requests sent: 77114
[basic][thread=1] Errors: 0% (0/77114)
[basic][thread=2] Mean Req/s: 19286.6074
[basic][thread=2] Mean Response time (ms): 0.0978
[basic][thread=2] Total requests sent: 77149
[basic][thread=2] Errors: 0% (0/77149)
[basic][thread=3] Mean Req/s: 19303.6245
[basic][thread=3] Mean Response time (ms): 0.0978
[basic][thread=3] Total requests sent: 77217
[basic][thread=3] Errors: 0% (0/77217)
[basic][thread=4] Mean Req/s: 19312.6944
[basic][thread=4] Mean Response time (ms): 0.0977
[basic][thread=4] Total requests sent: 77253
[basic][thread=4] Errors: 0% (0/77253)
[basic][global] Mean Req/s: 96461.7948
[basic][global] Mean Response Time (ms): 0.0978
[basic][global] Total requests sent: 385859
[basic][global] Errors: 0.00% (0/385859)
----Snapshot 5----
```

### llm bench (llamacpp)
```bash
cargo run --release -- http://localhost:58969/v1/chat/completions -m post --json '{"model": "Qwen3.6-35B-A3B-GGUF", "messages": [{"role": "user", "content": "hello. Who are you?"}]}' -t 2 -c 2 --plugins basic,llamacpp
```

Output:
```bash
----Snapshot 12----
[basic][thread=0] Mean Req/s: 0.4064
[basic][thread=0] Mean Response time (ms): 5123.7338
[basic][thread=0] Total requests sent: 4
[basic][thread=0] Errors: 0% (0/4)
[basic][thread=1] Mean Req/s: 0.6504
[basic][thread=1] Mean Response time (ms): 3899.6921
[basic][thread=1] Total requests sent: 3
[basic][thread=1] Errors: 0% (0/3)
[basic][global] Mean Req/s: 0.9755
[basic][global] Mean Response Time (ms): 4599.1445
[basic][global] Total requests sent: 7
[basic][global] Errors: 0.00% (0/7)
[llamacpp][thread=0] Mean Prompt Tok/s: 96.1152
[llamacpp][thread=0] Mean Completion Tok/s: 48.6212
[llamacpp][thread=0] Prompt Tok/req: 18.0000
[llamacpp][thread=0] Completion Tok/req: 73.5000
[llamacpp][thread=0] Prompt tokens: 72
[llamacpp][thread=0] Completion tokens: 294
[llamacpp][thread=0] Total tokens: 366
[llamacpp][thread=1] Mean Prompt Tok/s: 77.4732
[llamacpp][thread=1] Mean Completion Tok/s: 49.0376
[llamacpp][thread=1] Prompt Tok/req: 18.0000
[llamacpp][thread=1] Completion Tok/req: 48.3333
[llamacpp][thread=1] Prompt tokens: 54
[llamacpp][thread=1] Completion tokens: 145
[llamacpp][thread=1] Total tokens: 199
[llamacpp][global] Mean Prompt Tok/s: 176.2515
[llamacpp][global] Mean Completion Tok/s: 97.5993
[llamacpp][global] Prompt Tok/req: 18.0000
[llamacpp][global] Completion Tok/req: 62.7143
[llamacpp][global] Prompt tokens: 126
[llamacpp][global] Completion tokens: 439
[llamacpp][global] Total tokens: 565
----Snapshot 12----
```
