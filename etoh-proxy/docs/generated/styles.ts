export default `/* Import all CSS components */
/* CSS Variables for theming */
:root {
	--bg-primary: #1a1a1a;
	--bg-secondary: #2d2d2d;
	--text-primary: #ffffff;
	--text-secondary: #b3b3b3;
	--accent-color: #61dafb;
	--border-color: #404040;
	--code-bg: #2d2d2d;
	--endpoint-bg: #2d2d2d;
	--shadow-color: rgba(0, 0, 0, 0.3);
}
body {
	font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
	line-height: 1.6;
	margin: 0;
	padding: 0;
	color: var(--text-primary);
	background-color: var(--bg-secondary);
	min-height: 100vh;
}

/* Typography */
h1 {
	color: var(--text-primary);
	margin: 0;
	font-size: 2.5em;
}

h2 {
	color: var(--text-primary);
	border-bottom: 2px solid var(--border-color);
	font-size: 1.8em;
}


h4 {
	margin: 0;
}
/* Code elements */
code {
	background: var(--code-bg);
	border-radius: 4px;
	padding: 2px 6px;
	font-family: 'Monaco', 'Consolas', monospace;
	font-size: 0.9em;
	color: var(--accent-color);
	border: 1px solid var(--border-color);
}

pre {
	background: #f5f7f9;
	border-radius: 4px;
	padding: 15px;
	overflow-x: auto;
}
.endpoint {
	background: var(--endpoint-bg);
	border: 1px solid var(--border-color);
	margin: 20px 0;
	border-radius: 8px;
	overflow: hidden;
}

.endpoint h3 {
	margin-top: 0;
	color: #2c3e50;
}

.method {
	color: #0056b3;
	font-weight: bold;
}

.endpoint p {
	margin: 10px 0;
}

.endpoint-info {
	display: flex;
	align-items: center;
	flex: 1;
	/* Allow the info section to grow */
}

.endpoint-info .path {
	margin-right: 10px;
	white-space: nowrap;
	/* Prevent wrapping */
}

.endpoint-info .description {
	flex: 1;
	/* Allow the description to take up remaining space */
	min-width: 0;
	/* Allow the description to shrink below its content size */
}

.endpoint .required {
	color: #a93e3e;
	font-size: 1rem;
	vertical-align: super;
}

.endpoint .required .star {
	font-size: 1.4rem;
}

.endpoint-header {
	padding: 0;
	background: var(--bg-secondary);
	position: relative;
	border-bottom: 1px solid var(--border-color);
	/* Add a minimum height to the header */
	min-height: 50px;
}

.endpoint-content {
	padding: 20px;
	display: none;
}

.endpoint-toggle:checked~.endpoint-content {
	display: block;
}

.method {
	padding: 5px 10px;
	border-radius: 4px;
	font-weight: bold;
	text-transform: uppercase;
	font-size: 0.9em;
}

.method.get {
	background: #61affe;
	color: white;
}

.method.post {
	background: #49cc90;
	color: white;
}

.method.put {
	background: #fca130;
	color: white;
}

.method.delete {
	background: #f93e3e;
	color: white;
}

.path {
	margin: 0;
}

table {
	/* width: 100%; */
	border-collapse: collapse;
	margin: 10px 0;
}

th,
td {
	padding: 0 10px;
	text-align: left;
	/* border: 1px solid var(--border-color); */
}

th {
	background: var(--bg-secondary);
	font-weight: 600;
}

.response {
	margin: 10px 0;
	border: 1px solid var(--border-color);
	border-radius: 4px;
}

.response-header {
	padding: 10px;
	background: var(--bg-secondary);
	display: flex;
	gap: 10px;
	align-items: center;
}

.status-code {
	font-weight: bold;
	vertical-align: top;
	text-align: center;
	font-size: 1.2rem;
}

.response-model {
	padding: 10px;
}

.response-model {

	td,
	th {
		border: 1px solid var(--border-color);
	}
}

.description {
	margin: 10px 0 20px 0;
	color: var(--text-secondary);
}

.endpoint-toggle {
	display: none;
}

.endpoint-toggle-button {
	position: relative;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	cursor: pointer;
	display: flex;
	align-items: center;
	justify-content: space-between;
	padding: 15px;
	box-sizing: border-box;
}

.expand-icon,
.collapse-icon {
	font-size: 1em;
}

.collapse-icon {
	display: none;
}

.endpoint-toggle:checked+.endpoint-toggle-button {
	height: auto;
	padding: 15px 15px 0px 15px;
}

.endpoint-toggle:checked+.endpoint-toggle-button .expand-icon {
	display: none;
}

.endpoint-toggle:checked+.endpoint-toggle-button .collapse-icon {
	display: inline-block;
}

.method,
.endpoint-info,
.expand-icon,
.collapse-icon {
	pointer-events: none;
}

.endpoint-toggle-button {
	pointer-events: auto;
}
.container {
	/* max-width: 1200px; */
	margin: 0 auto;
	padding: 20px;
	background-color: var(--bg-primary);
	box-shadow: 0 2px 4px var(--shadow-color);
	border-radius: 8px;
}

.header-row {
	position: fixed;
	text-align: center;
	width: 100%;
	background-color: var(--bg-secondary);
	z-index: 1000;
}


/* Lists */
ul {
	padding-left: 20px;
}

ul li {
	margin: 8px 0;
	color: var(--text-secondary);
}

/* Sections */
section {
	/* margin: 40px 0; */
}

#introduction {
	font-size: 1.1em;
	color: var(--text-secondary);
	/* margin-bottom: 40px; */
}

#introduction p {
	margin: 0;
	text-align: center;
	color: white;
	font-size: 1.2rem;
}

#introduction div {
	text-align: center;
}

/* Responsive adjustments */
@media (max-width: 768px) {
	.sidebar {
		width: 100%;
		height: auto;
		position: relative;
		padding: 10px;
	}

	.content {
		margin-left: 0;
		padding: 10px;
	}

	.container {
		margin: 0;
		padding: 15px;
		border-radius: 0;
	}

	h1 {
		font-size: 2em;
	}

	.endpoint {
		padding: 15px;
	}

	.nav-content {
		margin-top: 20px;
	}

	.theme-toggle {
		position: absolute;
	}
}
/* Add scroll margin to all sections to account for fixed header */
section {
	/* Adjust this value to match your header height */
	scroll-margin-top: 5rem;
}

/* Also add to any other elements that might be scroll targets */
[id] {
	scroll-margin-top: 5rem;
}

/* Navigation */
.nav-content {
	margin-top: 60px;
}

.nav-content ul {
	list-style: none;
	padding: 0;
}

.nav-content a {
	color: var(--text-primary);
	text-decoration: none;
	padding: 8px 0;
	display: block;
	transition: color 0.3s;
}

.nav-content a:hover {
	color: var(--accent-color);
}


/* Endpoint navigation */
.endpoint-nav {
	padding-left: 15px !important;
	margin-top: 5px !important;
	font-size: 0.9em;
}

.endpoint-nav li {
	margin: 4px 0 !important;
}

.endpoint-nav a {
	color: var(--text-secondary) !important;
	padding: 4px 0 !important;
}

.endpoint-nav a:hover {
	color: var(--accent-color) !important;
}


/* Sidebar + sidebar menu */
.sidebar {
	position: fixed;
	top: 0;
	/* Start off-screen */
	left: -250px;
	width: 250px;
	height: 100vh;
	background: linear-gradient(to right, var(--bg-primary) 50%, transparent);
	/* border-right: 1px solid var(--border-color); */
	padding: 20px;
	box-sizing: border-box;
	transition: left 0.3s ease-in-out;
	z-index: 100;
}

/* When menu is checked, show sidebar */
.menu-toggle input:checked~.sidebar {
	left: 0;
}

.menu-toggle .hamburger-container h3 {
	transition: opacity 0.3s ease-in-out;
}

.menu-toggle input:checked~.hamburger-container h3 {
	opacity: 1;
}

.menu-toggle input:not(:checked)~.hamburger-container h3 {
	opacity: 0;
}


.menu-toggle {
	position: relative;
	display: inline-block;
	width: 60px;
	height: 60px;
	cursor: pointer;
}

/* New container styles */
.menu-toggle .hamburger-container {
	display: flex;
	align-items: center;
	height: 60px;
	/* Adjust spacing as needed */
	padding-left: 20px;
	position: fixed;
	z-index: 1000;
}

.menu-toggle .hamburger-container h3 {
	margin: 0;
	/* Space between hamburger and text */
	padding-left: 10px;
	font-size: 1.2em;
}

.menu-toggle input {
	opacity: 0;
	width: 0;
	height: 0;
	position: fixed;
}

.hamburger {
	position: relative;
	width: 30px;
	height: 20px;
	z-index: 1000;
}

.hamburger span {
	position: absolute;
	width: 100%;
	height: 3px;
	background-color: #555;
	border-radius: 3px;
	transition: all 0.3s ease-in-out;
}

/* The three lines of the hamburger icon */
.hamburger span:nth-child(1) {
	top: 0;
}

.hamburger span:nth-child(2) {
	top: 50%;
	transform: translateY(-50%);
}

.hamburger span:nth-child(3) {
	bottom: 0;
}

/* Animation when checked */
input:checked+.hamburger span:nth-child(1) {
	transform: translateY(9px) rotate(45deg);
}

input:checked+.hamburger span:nth-child(2) {
	opacity: 0;
}

input:checked+.hamburger span:nth-child(3) {
	transform: translateY(-9px) rotate(-45deg);
}

/* Optional: Hover effect */
.menu-toggle:hover .hamburger span {
	background-color: #888;
}
`;