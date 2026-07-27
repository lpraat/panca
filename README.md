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
----Snapshot 6----
[basic][thread=0] Req/s: 19178.7062 ± 360.1154 [min=18449.7640, max=19439.7805]
[basic][thread=0] Response time (ms): 0.0992 ± 0.0001 [min=0.0553, max=2.3611]
[basic][thread=0] Total requests sent: 95895
[basic][thread=0] Errors: 0% (0/95895)
[basic][thread=1] Req/s: 19192.8991 ± 369.3096 [min=18445.8432, max=19465.3398]
[basic][thread=1] Response time (ms): 0.0992 ± 0.0001 [min=0.0554, max=2.3415]
[basic][thread=1] Total requests sent: 95966
[basic][thread=1] Errors: 0% (0/95966)
[basic][thread=2] Req/s: 19216.7853 ± 348.3238 [min=18508.1579, max=19435.7765]
[basic][thread=2] Response time (ms): 0.0989 ± 0.0001 [min=0.0520, max=2.2262]
[basic][thread=2] Total requests sent: 96087
[basic][thread=2] Errors: 0% (0/96087)
[basic][thread=3] Req/s: 19181.9794 ± 346.8607 [min=18476.8707, max=19412.6239]
[basic][thread=3] Response time (ms): 0.0992 ± 0.0001 [min=0.0535, max=2.2072]
[basic][thread=3] Total requests sent: 95911
[basic][thread=3] Errors: 0% (0/95911)
[basic][thread=4] Req/s: 19169.2204 ± 377.5314 [min=18401.2333, max=19416.2250]
[basic][thread=4] Response time (ms): 0.0992 ± 0.0001 [min=0.0568, max=2.2358]
[basic][thread=4] Total requests sent: 95849
[basic][thread=4] Errors: 0% (0/95849)
[basic][global] Req/s: 95939.5905 ± 736.8399
[basic][global] Response time (ms): 0.0992 ± 0.0001 [min=0.0520, max=2.3611]
[basic][global] Total requests sent: 479708
[basic][global] Errors: 0.00% (0/479708)
----Snapshot 6----
```

### llm bench (llamacpp)
```bash
cargo run --release -- http://localhost:58969/v1/chat/completions -m post --json '{"model": "Qwen3.6-35B-A3B-GGUF", "messages": [{"role": "user", "content": "hello. Who are you?"}]}' -t 2 -c 2 --plugins basic,llamacpp
```

Output:
```bash
----Snapshot 12----
[basic][thread=0] Req/s: 0.4774 ± 0.2212 [min=0.2613, max=0.6420]
[basic][thread=0] Response time (ms): 3941.2198 ± 608.3542 [min=3115.4238, max=4494.4673]
[basic][thread=0] Total requests sent: 4
[basic][thread=0] Errors: 0% (0/4)
[basic][thread=1] Req/s: 0.4972 ± 0.2589 [min=0.3104, max=0.8799]
[basic][thread=1] Response time (ms): 3946.8388 ± 1252.7297 [min=2142.2233, max=5146.3762]
[basic][thread=1] Total requests sent: 4
[basic][thread=1] Errors: 0% (0/4)
[basic][global] Req/s: 0.9775 ± 0.3237
[basic][global] Response time (ms): 3944.0293 ± 644.6671 [min=2142.2233, max=5146.3762]
[basic][global] Total requests sent: 8
[basic][global] Errors: 0.00% (0/8)
[llamacpp][thread=0] Prompt Tok/s: 96.1411 ± 0.6468 [min=95.1950, max=96.6254]
[llamacpp][thread=0] Completion Tok/s: 48.6428 ± 0.5147 [min=48.1265, max=49.1820]
[llamacpp][thread=0] Prompt Tok/req: 18.0000
[llamacpp][thread=0] Completion Tok/req: 57.0000
[llamacpp][thread=0] Prompt tokens: 72
[llamacpp][thread=0] Completion tokens: 228
[llamacpp][thread=0] Total tokens: 300
[llamacpp][thread=1] Prompt Tok/s: 72.2912 ± 45.4444 [min=2.7508, max=96.3206]
[llamacpp][thread=1] Completion Tok/s: 48.9200 ± 0.3387 [min=48.4128, max=49.1703]
[llamacpp][thread=1] Prompt Tok/req: 18.0000
[llamacpp][thread=1] Completion Tok/req: 35.7500
[llamacpp][thread=1] Prompt tokens: 72
[llamacpp][thread=1] Completion tokens: 143
[llamacpp][thread=1] Total tokens: 215
[llamacpp][global] Prompt Tok/s: 168.4323 ± 45.6365
[llamacpp][global] Completion Tok/s: 97.5628 ± 0.6063
[llamacpp][global] Prompt Tok/req: 18.0000
[llamacpp][global] Completion Tok/req: 46.3750
[llamacpp][global] Prompt tokens: 144
[llamacpp][global] Completion tokens: 371
[llamacpp][global] Total tokens: 515
----Snapshot 12----
```
