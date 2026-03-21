#!/bin/bash
cd "$(dirname "$0")"
source venv/bin/activate
curl -s http://127.0.0.1:8080/health >/dev/null || (echo "Start server first!" && exit 1)
python agent.py "$@"
