import { ENDPOINTS } from './endpoints';

import template from './generated/template';
import styles from './generated/styles';
import { endpoint, endpoint_parameter, endpoint_response, endpoint_response_model, endpoint_response_model_sub } from './generated/endpoints';

function generateEndpointCategories() {
	let navItems = '';
	let html = '';

	Object.entries(ENDPOINTS).forEach(([category, endpoints]) => {
		const categoryId = category.toLowerCase();
		html += `<section id="${categoryId}"><h2>${category}</h2>`;
		html += `<ul>`;
		html += endpoints.map(endpoint => generateEndpointHTML(endpoint)).join('\n');
		html += `</ul></section>`;

		navItems += `<li><a href="#${categoryId}">${category}</a></li>`;
	});

	return {
		html,
		navItems
	};
}

function generateEndpointHTML(endpoint_docs: Endpoint) {
	const id = `endpoint-${endpoint_docs.method.toLowerCase()}-${endpoint_docs.path.replace(/[^a-zA-Z0-9]/g, '-')}`;

	let parameters = endpoint_docs.parameters.map(param => endpoint_parameter
		.replace(/{{name}}/g, param.name)
		.replace(/{{type}}/g, param.type)
		.replace(/{{description}}/g, param.description)
		.replace(/{{required}}/g, param.required ? '<span class="required"><span class="star">*</span>required</span>' : '')
		.replace(/{{notes}}/g, param.notes ? '<ul>' + param.notes.map(note => `<li>${note}</li>`).join('') + '</ul>' : '')
	).join('\n');

	endpoint_docs.responses.sort((a, b) => a.code < b.code ? -1 : 1);
	let responses = endpoint_docs.responses.map(response => endpoint_response
		.replace(/{{code}}/g, response.code.toString())
		.replace(/{{description}}/g, response.description)
		.replace(/{{model}}/g, response.model ? endpoint_response_model
			.replace(/{{model-sub}}/g, Object.entries(response.model).map(([field, model]) => endpoint_response_model_sub
				.replace(/{{field}}/g, field)
				.replace(/{{type}}/g, model.type)
				.replace(/{{description}}/g, model.description)
			).join('\n')) : ''
		)
	).join('\n');

	return endpoint
		.replaceAll(/{{id}}/g, id)
		.replace(/{{method}}/g, endpoint_docs.method)
		.replace(/{{method_l}}/g, endpoint_docs.method.toLowerCase())
		.replace(/{{path}}/g, endpoint_docs.path)
		.replace(/{{description}}/g, endpoint_docs.description)
		.replace(/{{parameters}}/g, parameters)
		.replace(/{{responses}}/g, responses);
}

export function generateDocumentation() {
	let { html, navItems } = generateEndpointCategories();

	return template
		.replace(/{{styles}}/g, styles)
		.replace(/{{endpoints}}/g, html)
		.replace(/{{endpoint-nav}}/g, navItems);
}
