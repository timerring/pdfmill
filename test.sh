#!/bin/bash

# PDFMill API Test Script
# Usage: ./test.sh [server_url]

SERVER_URL="${1:-http://localhost:3000}"
EXAMPLES_DIR="examples"

echo "🧪 Testing PDFMill API at $SERVER_URL"
echo "========================================"

# Test 1: Health Check
echo ""
echo "1️⃣ Testing health endpoint..."
curl -s "$SERVER_URL/health" | jq . || echo "❌ Health check failed"

# Test 2: Service Info
echo ""
echo "2️⃣ Testing info endpoint..."
curl -s "$SERVER_URL/info" | jq '.supported_formats' || echo "❌ Info endpoint failed"

# Test 3: Convert HTML
echo ""
echo "3️⃣ Testing HTML conversion..."
if [ -f "$EXAMPLES_DIR/sample.html" ]; then
    curl -X POST "$SERVER_URL/convert" \
      -F "file=@$EXAMPLES_DIR/sample.html" \
      -F "printBackground=true" \
      -o test_output_html.pdf \
      --silent --show-error
    
    if [ -f "test_output_html.pdf" ] && [ -s "test_output_html.pdf" ]; then
        echo "✅ HTML → PDF conversion successful ($(wc -c < test_output_html.pdf) bytes)"
    else
        echo "❌ HTML → PDF conversion failed"
    fi
else
    echo "⚠️  Sample HTML file not found"
fi

# Test 4: Convert Markdown
echo ""
echo "4️⃣ Testing Markdown conversion..."
if [ -f "$EXAMPLES_DIR/sample.md" ]; then
    curl -X POST "$SERVER_URL/convert" \
      -F "file=@$EXAMPLES_DIR/sample.md" \
      -o test_output_md.pdf \
      --silent --show-error
    
    if [ -f "test_output_md.pdf" ] && [ -s "test_output_md.pdf" ]; then
        echo "✅ Markdown → PDF conversion successful ($(wc -c < test_output_md.pdf) bytes)"
    else
        echo "❌ Markdown → PDF conversion failed"
    fi
else
    echo "⚠️  Sample Markdown file not found"
fi

# Test 5: Test with landscape option
echo ""
echo "5️⃣ Testing with landscape option..."
if [ -f "$EXAMPLES_DIR/sample.html" ]; then
    curl -X POST "$SERVER_URL/convert" \
      -F "file=@$EXAMPLES_DIR/sample.html" \
      -F "landscape=true" \
      -o test_output_landscape.pdf \
      --silent --show-error
    
    if [ -f "test_output_landscape.pdf" ] && [ -s "test_output_landscape.pdf" ]; then
        echo "✅ Landscape conversion successful ($(wc -c < test_output_landscape.pdf) bytes)"
    else
        echo "❌ Landscape conversion failed"
    fi
fi

# Test 6: Test unsupported format
echo ""
echo "6️⃣ Testing unsupported format handling..."
echo "test" > /tmp/test.xyz
curl -X POST "$SERVER_URL/convert" \
  -F "file=@/tmp/test.xyz" \
  -o /dev/null \
  -w "HTTP Status: %{http_code}\n" \
  --silent
rm -f /tmp/test.xyz

echo ""
echo "========================================"
echo "✅ Tests completed!"
echo ""
echo "Generated files:"
ls -lh test_output_*.pdf 2>/dev/null || echo "No PDF files generated"
