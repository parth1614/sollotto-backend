const dotenv = require('dotenv');
dotenv.config();
module.exports = {
    MONGODB:
      `mongodb://${process.env.MONGO_USERNAME}:${process.env.MONGO_PASS}@cluster0-shard-00-00.g0kih.mongodb.net:27017,cluster0-shard-00-01.g0kih.mongodb.net:27017,cluster0-shard-00-02.g0kih.mongodb.net:27017/Sollotto-offchain?ssl=true&replicaSet=atlas-8ehri1-shard-0&authSource=admin&retryWrites=true&w=majority`,
    CHARITY_STATUS :{
          IN_QUE: "IN QUE",
          VOTE_NOW:"VOTE NOW",
          NOT_ELIGIBLE:"NOT ELIGIBLE"
    }
    }

