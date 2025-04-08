const fs = require('fs');
const path = require('path');

// Read the content of template.html and styles.css and save them as JavaScript modules
function buildDocs() {
	const templatePath = path.join(__dirname, '../docs/template.html');
	const stylePath = path.join(__dirname, '../docs/styles.css');

	const template = fs.readFileSync(templatePath, 'utf8');
	const styles = fs.readFileSync(stylePath, 'utf8');

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

	// Write styles.ts
	fs.writeFileSync(
		path.join(generatedPath, 'styles.ts'),
		`export default \`${styles.replace(/\`/g, '\\`')}\`;`
	);
}

buildDocs();
