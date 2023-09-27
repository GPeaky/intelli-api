@0xccf601c96a055c17;

# LapHistoryData
struct LapHistoryData {
    lapTimeInMS @0 :UInt32;
    sector1TimeInMS @1 :UInt16;
    sector1TimeMinutes @2 :UInt8;
    sector2TimeInMS @3 :UInt16;
    sector2TimeMinutes @4 :UInt8;
    sector3TimeInMS @5 :UInt16;
    sector3TimeMinutes @6 :UInt8;
    lapValidBitFlags @7 :UInt8;
}

# TyreStintHistoryData
struct TyreStintHistoryData {
    endLap @0 :UInt8;
    tyreActualCompound @1 :UInt8;
    tyreVisualCompound @2 :UInt8;
}

# PacketSessionHistoryData
struct PacketSessionHistoryData {
    carIdx @0 :UInt8;
    numLaps @1 :UInt8;
    numTyreStints @2 :UInt8;
    bestLapTimeLapNum @3 :UInt8;
    bestSector1LapNum @4 :UInt8;
    bestSector2LapNum @5 :UInt8;
    bestSector3LapNum @6 :UInt8;
    lapHistoryData @7 :List(LapHistoryData);
    tyreStintsHistoryData @8 :List(TyreStintHistoryData);
}
