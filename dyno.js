var http = require('http'); //importing http

var options = {
    host: 'sollotto-backend.herokuapp.com/',
    port: 80,
    path: '/WAKEUP_DYNO'
};
console.log("======WAKUP DYNO START");
http.get(options, function(res) {
    res.on('data', function() {
        try {
            // optional logging... disable after it's working
            console.log("======WAKUP DYNO: HEROKU RESPONSE:");
        } catch (err) {
            console.log(err.message);
        }
    });
}).on('error', function(err) {
    console.log("Error: " + err.message);
});
