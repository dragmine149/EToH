import { ENDPOINTS } from './endpoints';

function generateEndpointCategories() {
	let navItems = '';
	let html = '';

	Object.entries(ENDPOINTS).forEach(([category, endpoints]) => {
		html += `<h2>${category}</h2>`;
		html += `<ul>`;
		html += endpoints.map(generateEndpointHTML).join('\n');
		html += `</ul>`;

		navItems += `
        <li><a href="#${category.toLowerCase()}">${category}</a></li>
    `;
	});

	return {
		html,
		navItems
	};
}

function generateEndpointHTML(endpoint: any) {
	const id = `endpoint-${endpoint.method.toLowerCase()}-${endpoint.path.replace(/[^a-zA-Z0-9]/g, '-')}`;
	return `
        <div class="endpoint" id="${id}">
            <div class="endpoint-header">
                <span class="method ${endpoint.method.toLowerCase()}">${endpoint.method}</span>
                <code class="path">${endpoint.path}</code>
                <button class="toggle-btn" onclick="toggleEndpoint('${id}')">▼</button>
            </div>
            <div class="endpoint-content">
                <p class="description">${endpoint.description}</p>

                ${endpoint.parameters?.length ? `
                <div class="parameters">
                    <h4>Parameters</h4>
                    <table>
                        <thead>
                            <tr>
                                <th>Name</th>
                                <th>Type</th>
                                <th>Description</th>
                                <th>Required</th>
                            </tr>
                        </thead>
                        <tbody>
                            ${endpoint.parameters.map(param => `
                                <tr>
                                    <td>${param.name}</td>
                                    <td><code>${param.type}</code></td>
                                    <td>${param.description}</td>
                                    <td>${param.required ? '✓' : ''}</td>
                                </tr>
                            `).join('')}
                        </tbody>
                    </table>
                </div>
                ` : ''}

                <div class="responses">
                    <h4>Responses</h4>
                    ${endpoint.responses.map(response => `
                        <div class="response">
                            <div class="response-header">
                                <span class="status-code">${response.code}</span>
                                <span class="status-text">${response.description}</span>
                            </div>
                            ${response.model ? `
                                <div class="response-model">
                                    <h5>Model</h5>
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>Field</th>
                                                <th>Type</th>
                                                <th>Description</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            ${Object.entries(response.model).map(([field, details]: [string, any]) => `
                                                <tr>
                                                    <td>${field}</td>
                                                    <td><code>${details.type}</code></td>
                                                    <td>${details.description}</td>
                                                </tr>
                                            `).join('')}
                                        </tbody>
                                    </table>
                                </div>
                            ` : ''}
                        </div>
                    `).join('')}
                </div>
            </div>
        </div>`;
}

export function generateDocumentation(template: string, styles: string) {
	let { html, navItems } = generateEndpointCategories();

	return template
		.replace('{{styles}}', styles)
		.replace('{{endpoints}}', html)
		.replace('{{endpoint-nav}}', navItems);
}
