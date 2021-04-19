
# Aggietime

Aggietime is Utah State University's time clock for tracking hours worked for hourly employees. This project is a webserver built in rust on top of the Rocket web framework that exposes and simplifies essential parts of the Aggietime API and parses web tokens from the HTML to fulfill the requests. Essentially this webserver is a man in the middle to parse tokens embedded in the html and re-expose the REST API in a simple and accessible way. 