@0xc44123e9d3f74b3a;

struct ParticipantData {
    aiControlled @0 :UInt8; # Whether the vehicle is AI (1) or Human (0) controlled
    driverId @1 :UInt8;     # Driver id - see appendix, 255 if network human
    networkId @2 :UInt8;    # Network id – unique identifier for network players
    teamId @3 :UInt8;       # Team id - see appendix
    myTeam @4 :UInt8;       # My team flag – 1 = My Team, 0 = otherwise
    raceNumber @5 :UInt8;   # Race number of the car
    nationality @6 :UInt8;  # ParticipantNationality // Nationality of the driver
    name @7 :Text;           # Name of participant in UTF-8 format – null terminated, Will be truncated with … (U+2026) if too long
    yourTelemetry @8 :UInt8; # The player's UDP setting, 0 = restricted, 1 = public
    showOnlineNames @9 :UInt8; # The player's show online names setting, 0 = off, 1 = on
    platform @10 :UInt8;      # 1 = Steam, 3 = PlayStation, 4 = Xbox, 6 = Origin, 255 = unknown
}

struct PacketParticipantsData {
    numActiveCars @0 :UInt8;
    participants @1 :List(ParticipantData);
}
