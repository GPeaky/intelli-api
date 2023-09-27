@0xbd7c2f2a4d93a9f9;

struct PacketEventData {
    eventStringCode @0 :Data; # Use Data for bytes
    eventDetails @1 :EventDataDetails;
}

struct EventDataDetails {
    union {
        fastestLap @0 :FastestLap;
        retirement @1 :Retirement;
        teamMateInPits @2 :TeamMateInPits;
        raceWinner @3 :RaceWinner;
        penalty @4 :Penalty;
        speedTrap @5 :SpeedTrap;
        startLights @6 :StartLights;
        driveThroughPenaltyServed @7 :DriveThroughPenaltyServed;
        stopGoPenaltyServed @8 :StopGoPenaltyServed;
        flashback @9 :Flashback;
        buttons @10 :Buttons;
        overtake @11 :Overtake;
    }
}

struct FastestLap {
    vehicleIdx @0 :UInt8;
    lapTime @1 :Float32;
}

struct Retirement {
    vehicleIdx @0 :UInt8;
}

struct TeamMateInPits {
    vehicleIdx @0 :UInt8;
}

struct RaceWinner {
    vehicleIdx @0 :UInt8;
}

struct Penalty {
    penaltyType @0 :UInt8;
    infringementType @1 :UInt8;
    vehicleIdx @2 :UInt8;
    otherVehicleIdx @3 :UInt8;
    time @4 :UInt8;
    lapNum @5 :UInt8;
    placesGained @6 :UInt8;
}

struct SpeedTrap {
    vehicleIdx @0 :UInt8;
    speed @1 :Float32;
    isOverallFastestInSession @2 :UInt8;
    isDriverFastestInSession @3 :UInt8;
    fastestVehicleIdxInSession @4 :UInt8;
    fastestSpeedInSession @5 :Float32;
}

struct StartLights {
    numLights @0 :UInt8;
}

struct DriveThroughPenaltyServed {
    vehicleIdx @0 :UInt8;
}

struct StopGoPenaltyServed {
    vehicleIdx @0 :UInt8;
}

struct Flashback {
    flashbackFrameIdentifier @0 :UInt32;
    flashbackSessionTime @1 :Float32;
}

struct Buttons {
    buttonStatus @0 :UInt32;
}

struct Overtake {
    overtakingVehicleIdx @0 :UInt8;
    beingOvertakenVehicleIdx @1 :UInt8;
}
