#!/bin/bash
set -e

echo "Generating clients for all programs..."

echo "Generating sol-mind-protocol clients..."
npx codama run --all -c codama/scripts/capital-program.json

echo "Generating nft-operations clients..."
npx codama run --all -c codama/scripts/nft-program.json

echo "All clients generated successfully!"
