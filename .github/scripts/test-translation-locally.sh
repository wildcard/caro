#!/bin/bash
#
# Local Translation Test Script
# Tests the translation workflow without GitHub Actions
#
# Usage:
#   export OPENAI_API_KEY="sk-..."
#   ./test-translation-locally.sh es
#
# Arguments:
#   $1 - Target locale (es, fr, pt, de, he, ar, uk, ru, ja, ko, hi, ur, fil, id)
#

set -e

# Check if locale is provided
if [ -z "$1" ]; then
  echo "Error: No target locale specified"
  echo "Usage: $0 <locale>"
  echo "Example: $0 es"
  exit 1
fi

# Check if OPENAI_API_KEY is set
if [ -z "$OPENAI_API_KEY" ]; then
  echo "Error: OPENAI_API_KEY environment variable is not set"
  echo "Please set it with: export OPENAI_API_KEY=\"sk-...\""
  exit 1
fi

TARGET_LOCALE="$1"

echo "========================================="
echo "Local Translation Test"
echo "========================================="
echo "Target Locale: $TARGET_LOCALE"
echo "Force Retranslate: ${FORCE_RETRANSLATE:-false}"
echo ""

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
  echo "Error: Node.js is not installed"
  exit 1
fi

# Check if openai package is installed locally
if [ ! -d "node_modules/openai" ]; then
  echo "Installing OpenAI package..."
  npm install openai
fi

# Run the translation script
echo "Running translation script..."
export TARGET_LOCALE="$TARGET_LOCALE"
export FORCE_RETRANSLATE="${FORCE_RETRANSLATE:-false}"

node .github/scripts/translate.js

echo ""
echo "========================================="
echo "Translation Test Completed!"
echo "========================================="
echo ""
echo "Translated files are in: website/src/i18n/locales/$TARGET_LOCALE/"
echo ""
echo "Next steps:"
echo "1. Review the translated JSON files"
echo "2. Test the website with: cd website && npm run dev"
echo "3. Visit http://localhost:4321/$TARGET_LOCALE/ to test"
echo ""
