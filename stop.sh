#!/bin/sh
PORT=9921

pid=$(lsof -iTCP:"$PORT" -sTCP:LISTEN -t 2>/dev/null)

if [ -z "$pid" ]; then
  echo "robost agent は起動していません"
  exit 0
fi

kill "$pid"
echo "robost agent を停止しました (PID $pid)"
