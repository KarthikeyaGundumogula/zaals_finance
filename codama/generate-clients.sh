#!/bin/bash
set -e

echo "building.."
anchor build

echo "Generating clients for programs..."

echo "Generating capital-program clients..."
npx codama run --all -c codama/scripts/capital-program.json

echo "Generating nft-program clients..."
npx codama run --all -c codama/scripts/nft-program.json

echo "Clients generated successfully!"
