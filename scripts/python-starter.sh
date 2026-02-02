#!/bin/bash
# MoFA Studio Python 启动脚本 - 设置正确的Python路径

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESOURCES_DIR="$(dirname "$SCRIPT_DIR")"

# 设置Python路径
PY_FRAMEWORK="$RESOURCES_DIR/python/Python.framework"
PY_VERSION="3.11"
PY_BIN="$PY_FRAMEWORK/Versions/Current/bin/python3"
SITE_PACKAGES="$PY_FRAMEWORK/Versions/Current/lib/python$PY_VERSION/site-packages"

# 导出环境变量
export PYTHONHOME="$PY_FRAMEWORK/Versions/Current"
export PYTHONPATH="$SITE_PACKAGES:$PYTHONPATH"

# 运行Python
exec "$PY_BIN" "$@"
