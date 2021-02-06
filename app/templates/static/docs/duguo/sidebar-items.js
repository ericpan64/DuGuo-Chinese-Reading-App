initSidebarItems({"fn":[["connect_to_mongodb","Connects to MongoDB (locally: Docker Container, in production: mongoDB Atlas). Connection is handled in main.rs."],["convert_rawstr_to_string","Sanitizes user input. Chinese punctuation is unaffected by this."],["launch_rocket","Starts the Rocket web server and corresponding services. Called in main.rs. Note: the Tokio version is deliberately set to 0.2.24 to match the MongoDB 1.1.1 driver. No new Tokio runtimes should be created in other functions and since they can lead to runtime panics."],["scrape_text_from_url","Scrapes relevant text from HTML. Returns: (Title, Body Text)."]],"mod":[["auth","Module handling user authentication and cookies."],["config","Module with config &str values."],["html","Module that handles HTML rendering. Often used with alias \"html_rendering\"."],["models","Module defining all data structures and associated functions."],["routes","Module defining all of the Rocket web endpoints."]],"trait":[["CacheItem","An object that can be found in Redis (using a uid)."],["DatabaseItem","An object that can be found in MongoDB. All DatabaseItem object fields are stored into MongoDB as String fields."]]});