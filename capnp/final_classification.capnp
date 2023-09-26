@0xeff4d3c1a09d76e7;

struct FinalClassificationData {
    position @0 :UInt8;
    numLaps @1 :UInt8;
    gridPosition @2 :UInt8;
    points @3 :UInt8;
    numPitStops @4 :UInt8;
    resultStatus @5 :UInt8;
    bestLapTimeInMS @6 :UInt32;
    totalRaceTime @7 :Float64; # Since the protobuf type is double
    penaltiesTime @8 :UInt8;
    numPenalties @9 :UInt8;
    numTyreStints @10 :UInt8;
    tyreStintsActual @11 :List(UInt8);
    tyreStintsVisual @12 :List(UInt8);
    tyreStintsEndLaps @13 :List(UInt8);
}

struct PacketFinalClassificationData {
    numCars @0 :UInt8;
    classificationData @1 :List(FinalClassificationData);
}
