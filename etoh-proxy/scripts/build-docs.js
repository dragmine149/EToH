const fs = require('fs');
const path = require('path');

// Read the content of template.html and styles.css and save them as JavaScript modules
function buildDocs() {
	const templatePath = path.join(__dirname, '../docs/templates/template.html');
	const docsPath = path.join(__dirname, '../docs/templates');
	const indexStylePath = path.join(__dirname, '../docs/styles/index.css');

	const template = fs.readFileSync(templatePath, 'utf8');

	// Find all endpoint template files
	const endpointFiles = fs.readdirSync(docsPath)
		.filter(file => file.startsWith('endpoint-') || file === 'endpoint.html')
		.map(file => ({
			name: file.replace('.html', ''),
			content: fs.readFileSync(path.join(docsPath, file), 'utf8')
		}));

	// Read index.css and find imports
	const indexCss = fs.readFileSync(indexStylePath, 'utf8');
	const importLines = indexCss.match(/^@import ['"](.+)['"];?$/gm) || [];
	const importPaths = importLines.map(line => {
		const match = line.match(/^@import ['"](.+)['"];?$/);
		return path.join(path.dirname(indexStylePath), match[1]);
	});

	// Build final CSS by combining index.css with imported files
	let styles = indexCss.replace(/^@import .+$\n*/gm, '');
	for (const importPath of importPaths) {
		styles += fs.readFileSync(importPath, 'utf8');
	}

	// Create generated folder if it doesn't exist
	const generatedPath = path.join(__dirname, '../docs/generated');
	if (!fs.existsSync(generatedPath)) {
		fs.mkdirSync(generatedPath);
	}

	// Write template.ts
	fs.writeFileSync(
		path.join(generatedPath, 'template.ts'),
		`export default \`${template.replace(/\`/g, '\\`')}\`;`
	);

	// Write endpoints.ts with all endpoint templates
	const endpointExports = endpointFiles
		.map(file => `export const ${file.name.replace(/-/g, '_')} = \`${file.content.replace(/\`/g, '\\`')}\`;`)
		.join('\n');
	fs.writeFileSync(
		path.join(generatedPath, 'endpoints.ts'),
		endpointExports
	);

	// Write styles.ts
	fs.writeFileSync(
		path.join(generatedPath, 'styles.ts'),
		`export default \`${styles.replace(/\`/g, '\\`')}\`;`
	);
}

buildDocs();
