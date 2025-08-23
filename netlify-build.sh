#!/bin/bash

# Netlify Build Script for Space Looter
# This script verifies that pre-built files exist (built by GitHub Actions)

set -e

echo "🚀 Netlify deployment for Space Looter..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}📋 Verifying pre-built files...${NC}"

# Check if required files exist (they should be pre-built by GitHub Actions)
if [ ! -f "index.html" ]; then
    echo -e "${RED}❌ index.html not found${NC}"
    echo -e "${YELLOW}This suggests the deploy branch is not set up correctly.${NC}"
    echo -e "${YELLOW}Files should be pre-built by GitHub Actions and pushed to the deploy branch.${NC}"
    exit 1
fi

if [ ! -f "space_looter.js" ]; then
    echo -e "${RED}❌ space_looter.js not found${NC}"
    echo -e "${YELLOW}This suggests the deploy branch is not set up correctly.${NC}"
    exit 1
fi

if [ ! -f "space_looter_bg.wasm" ]; then
    echo -e "${RED}❌ space_looter_bg.wasm not found${NC}"
    echo -e "${YELLOW}This suggests the deploy branch is not set up correctly.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All required files found!${NC}"

# Show file information
echo -e "${BLUE}📊 Deployment files:${NC}"
echo "   HTML: $(du -h index.html | cut -f1) - $(basename index.html)"
echo "   JS: $(du -h space_looter.js | cut -f1) - space_looter.js"
echo "   WASM: $(du -h space_looter_bg.wasm | cut -f1) - space_looter_bg.wasm"

# Check for optional files
if [ -f "DEPLOYMENT.md" ]; then
    echo "   Deployment info: DEPLOYMENT.md"
    echo -e "${BLUE}📝 Deployment details:${NC}"
    cat DEPLOYMENT.md
fi

if [ -d "assets" ]; then
    echo -e "${GREEN}✅ Assets directory found${NC}"
    echo "   Assets: $(du -sh assets | cut -f1) - game assets"
fi

echo ""
echo -e "${GREEN}🌐 Files ready for Netlify deployment!${NC}"
echo -e "${BLUE}ℹ️  Note: Files are pre-built by GitHub Actions${NC}"
echo -e "${BLUE}ℹ️  No compilation needed on Netlify${NC}"
