#!/bin/bash

# 设置证书 SHA-1 指纹
HASH=78055F15F4D6E7C85EF890EA066AACBCB1E908C8

echo "=== 步骤 1: 重新签名应用 ==="
codesign --deep --force --options runtime \
  --entitlements build/macos-full/entitlements.plist \
  -s "$HASH" "build/macos-full/MoFA Studio.app"

if [ $? -ne 0 ]; then
  echo "❌ 签名失败"
  exit 1
fi

echo "✅ 签名完成"
echo ""

echo "=== 步骤 2: 验证签名 ==="
codesign --verify --deep --strict --verbose=2 "build/macos-full/MoFA Studio.app"

if [ $? -ne 0 ]; then
  echo "❌ 签名验证失败"
  exit 1
fi

echo "✅ 签名验证通过"
echo ""

echo "=== 步骤 3: 创建 DMG ==="
# 先删除旧的 DMG（如果存在）
if [ -f "build/macos-full/MoFA-Studio-0.1.0.dmg" ]; then
  rm "build/macos-full/MoFA-Studio-0.1.0.dmg"
  echo "已删除旧的 DMG 文件"
fi

hdiutil create -volname "MoFA Studio" \
  -srcfolder "build/macos-full/MoFA Studio.app" \
  -ov -format UDZO "build/macos-full/MoFA-Studio-0.1.0.dmg"

if [ $? -ne 0 ]; then
  echo "❌ DMG 创建失败"
  exit 1
fi

echo "✅ DMG 创建成功"
echo ""

echo "=== 完成！==="
echo "DMG 文件位置: build/macos-full/MoFA-Studio-0.1.0.dmg"
