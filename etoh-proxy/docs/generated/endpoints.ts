export const endpoint_parameter = `<tr>
  <td><p>{{name}}{{required}}</p>
  	<code style="color: gray">{{type}}</code>
  </td>
  <td>{{description}}
  	{{notes}}
  </td>
</tr>
`;
export const endpoint_response_model_sub = `<tr>
  <td>{{field}}
  	<code>{{type}}</code>
  </td>
  <td>{{description}}</td>
</tr>
`;
export const endpoint_response_model = `<div class="response-model">
  <h5>Model</h5>
  <table>
    <thead>
      <tr>
        <th>Field</th>
        <th>Type</th>
      </tr>
    </thead>
    <tbody>
    	{{model-sub}}
    </tbody>
  </table>
</div>
`;
export const endpoint_response = `<tr class="response">
  <td class="status-code">{{code}}</td>
  <td class="status-text">{{description}}
	  {{model}}
  </td>
</tr>
`;
export const endpoint = `<div class="endpoint" id="{{id}}">
  <div class="endpoint-header">
    <input type="checkbox" id="toggle-{{id}}" class="endpoint-toggle">
    <label for="toggle-{{id}}" class="endpoint-toggle-button">
      <span class="method {{method_l}}">{{method}}</span>
      <div class="endpoint-info">
        <code class="path">{{path}}</code>
        <p class="description">{{description}}</p>
      </div>
      <span class="expand-icon">&#x25BC;</span>
      <span class="collapse-icon">&#x25B2;</span>
    </label>
	  <div class="endpoint-content">
	    <div class="parameters">
	      <h4>Parameters</h4>
	      <table>
	        <thead>
	          <tr>
	            <th>Name</th>
	            <th>Description</th>
	          </tr>
	        </thead>
	        <tbody>
	          {{parameters}}
	        </tbody>
	      </table>
	    </div>
	    <div class="responses">
	      <h4>Responses</h4>
				<table>
					<thead>
						<tr>
							<th>Code</th>
							<th>Description</th>
						</tr>
					</thead>
					<tbody>
						{{responses}}
					</tbody>
				</table>
	    </div>
	  </div>
	</div>
</div>
`;