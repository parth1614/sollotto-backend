const { ApolloServer, gql } = require("apollo-server-express");
const express = require("express");
const mongoose = require("mongoose");
const typeDefs = require("./typedefs/index");
const resolvers = require("./resolvers/index");
const { MONGODB } = require("./config");
const cron = require("node-cron");
const cors = require("cors");
const path = require("path");
const { changeDraw } = require("./utils/changeDraw");
const { resetDb } = require("./utils/resetDB");
const { uploadCharityImage } = require("./Routes/ImageUploadCharity");
const multer = require("multer");

async function startServer() {
  const app = express();
  const server = new ApolloServer({
    typeDefs,
    resolvers,
  });
  await server.start();
  app.use(cors());

  app.use("/static", express.static(path.join(__dirname, "public")));
  app.all("/", function (req, res, next) {
    res.header("Access-Control-Allow-Origin", "*");
    res.header("Access-Control-Allow-Headers", "X-Requested-With");
    next();
  });
  server.applyMiddleware({ app: app });
  mongoose.set("useFindAndModify", false);
  mongoose
    .connect(MONGODB, { useNewUrlParser: true, useUnifiedTopology: true })
    .then(() => {
      console.log(`MongoDb Connected`);

      console.log("inside cron then");
      cron.schedule(
        "0 0 * * wed,sat",
        () => {
          changeDraw();
        },
        {
          scheduled: true,
          timezone: "Atlantic/Azores",
        }
      );

      // .then(() => {
      //   console.log("inside cron then");
      //   cron.schedule("*/1 * * * *", () => {changeDraw()},
      //   {
      //     scheduled: true,
      //     timezone: "Atlantic/Azores"
      //   }
      //   );
      // })
    })
    .catch((err) => {
      console.log(err);
    });
  uploadCharityImage(app, multer);
  app.listen({ port: process.env.PORT || 5000 }, () => {
    console.log("server running at port 5000");
  });
}
startServer();
