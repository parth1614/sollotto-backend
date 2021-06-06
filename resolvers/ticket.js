const Ticket = require("../models/ticket");
const Lottery = require("../models/lottery");
module.exports = {
  Mutation: {
    async addTicket(
      _,
      { walletID, ticketArray, charityId, DataWallet, LotteryId },
      context,
      info
    ) {
     
      const newTicket = new Ticket({
        walletID,
        ticketArray,
        charityId,
        DataWallet,
        LotteryId,
      });
      const lottery = await Lottery.findOne({ Id: LotteryId });
      const updateLottery = await Lottery.findOneAndUpdate(
        { Id: LotteryId, "CharityVoteCount.charityId": charityId },
        {
          $inc: {
            TotalRegistrations: 1,
            TotalPoolValue: lottery.TicketPrice,
            "CharityVoteCount.$.votes": 1,
          },
        },
        { new: true }
      );
      const res = await newTicket.save();
      return "Ticket Saved Successfully";
    },
  },
  Query: {
     async getUserTickets(_,{walletID,LotteryId},context,info){
      
      const tickets = await Ticket.find({walletID:walletID, LotteryId:LotteryId})
     return tickets
      
    }
  },
};
