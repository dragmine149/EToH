import http.server
import socketserver
import json # Import the json module

PORT = 8080
LOG_FILE = "post_data.log"

class CustomHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def do_POST(self):
        if self.path == '/log':
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)

            try:
                # Parse the JSON data
                json_data = json.loads(post_data.decode('utf-8'))

                with open(LOG_FILE, "a") as f:
                    # f.write(f"--- New POST Request (JSON) ---\n")
                    # Use json.dumps for pretty printing the JSON data
                    f.write(json.dumps(json_data, indent=4))
                    f.write("\n")

                self.send_response(200)
                self.send_header("Content-type", "text/plain")
                self.end_headers()
                self.wfile.write(b"Data logged successfully.")

            except json.JSONDecodeError:
                # Handle cases where the data is not valid JSON
                self.send_response(400)
                self.send_header("Content-type", "text/plain")
                self.end_headers()
                self.wfile.write(b"Invalid JSON data.")

        else:
            # For other paths, return a 404
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"Not Found")

Handler = CustomHTTPRequestHandler

with socketserver.TCPServer(("", PORT), Handler) as httpd:
    print(f"Serving at port {PORT}")
    print(f"POST requests to /log will be saved to {LOG_FILE}")
    httpd.serve_forever()
