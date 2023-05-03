# Bottomless-Site
This is a webserver that I wrote for fun and use it to run my personal website which you can find at [here](turtlebamboo.com) assuming it hasnt crashed or gone down for some reason. It is very limited as it does not support sending (or reciving) most of the MIME data types; however, I did try to limit unwrap use to make the server as stable as possible.

---
The main control flow is as follows:
* A request is made
* it is then split into POST or GET requests
* POST request:
    * A post request is automatically considered to be an API request and it is handled like an api
    * the API is taken from the hashmap and executed
* GET request:
    * some path manipulation is done to determine the type of request
    * some "security" methods are added (really just making sure no one tries to ../../ out of the main directory)
    * the file is found metadata is read and the appropriate file is sent back
