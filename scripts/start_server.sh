#!/bin/bash
M="$HOME/gpt-oss-agent/models/OpenAI-20B-NEOPlus-Uncensored-IQ4_NL.gguf"
[ ! -f "$M" ] && echo "Download model first!" && exit 1
cd "$(dirname "$0")"

./llama.cpp/build/bin/llama-server \
    -m "$M" \
    -ngl 6 \
    -c 8192 \
    -t 6 \
    -b 512 \
    -ub 256 \
    -fa on \
    --host 127.0.0.1 \
    --port 8080
