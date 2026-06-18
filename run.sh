#!/bin/sh
set -e

PORT=9921
URL="http://localhost:$PORT"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# すでにエージェントが起動中ならブラウザを開くだけ
if lsof -iTCP:"$PORT" -sTCP:LISTEN -t >/dev/null 2>&1; then
  echo "robost agent はすでに起動しています ($URL)"
  open "$URL"
  exit 0
fi

# rpa バイナリを探す
if   [ -f "$SCRIPT_DIR/rpa" ];                        then RPA="$SCRIPT_DIR/rpa"
elif [ -f "$SCRIPT_DIR/target/release/rpa" ];         then RPA="$SCRIPT_DIR/target/release/rpa"
elif [ -f "$SCRIPT_DIR/target/debug/rpa" ];           then RPA="$SCRIPT_DIR/target/debug/rpa"
elif command -v rpa >/dev/null 2>&1;                  then RPA="rpa"
else
  echo "ERROR: rpa が見つかりません。"
  echo "先に 'cargo build --release -p robost-cli' でビルドしてください。"
  exit 1
fi

echo "robost agent を起動しています... ($URL)"
echo "終了するには Ctrl+C を押してください"
echo
"$RPA" agent --port "$PORT"
