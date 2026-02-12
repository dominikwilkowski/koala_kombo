#!/bin/bash
# build-wasm.sh - Build and serve Koala Kombo for web (fully local)

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RESET='\033[0m'

echo -e "${GREEN}=== Koala Kombo WASM Build ===${RESET}"

# 1. Check/add WASM target
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
	echo -e "${YELLOW}Adding wasm32-unknown-unknown target...${RESET}"
	rustup target add wasm32-unknown-unknown
fi

# 2. Build for WASM
echo -e "${GREEN}Building for WASM...${RESET}"
cargo build --lib --target wasm32-unknown-unknown --release

# 3. Create web directory
mkdir -p web

# 4. Run wasm-bindgen via cargo (no global install!)
echo -e "${GREEN}Generating JS bindings...${RESET}"
wasm-bindgen \
	--out-dir ./web \
	--target web \
	target/wasm32-unknown-unknown/release/koala_kombo.wasm

# 5. Copy data directory (resource registry)
cp -r ./data ./web/

# 6. Create index.html
cat > web/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
	<meta charset="utf-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>Koala Kombo</title>
	<style>
		* { margin: 0; padding: 0; }
		html, body {
			width: 100%;
			height: 100%;
			overflow: hidden;
			background: #000;
		}
		canvas {
			display: block;
			width: 100%;
			height: 100%;
			&:focus-visible {
				outline: none;
			}
		}
	</style>
</head>
<body>
	<script type="module">
		import init from './koala_kombo.js';

		async function run() {
			await init();
		}

		run().catch(console.error);
	</script>
</body>
</html>
EOF

echo -e "${GREEN}Starting web server...${RESET}"
echo -e "Open: ${YELLOW}http://localhost:8080${RESET}"
python3 -m http.server 8080 --directory ./web
