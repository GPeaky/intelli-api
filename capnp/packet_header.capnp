@0xb341a3e1a5d24e58;

struct PacketHeader {
    enum PacketType {
        carMotion @0;
        eventData @1;
        finalClassificationData @2;
        participants @3;
        sessionData @4;
        sessionHistoryData @5;
    }

    packetType @0 :PacketType;
    payload @1 :Data; # Use Data for bytes
}
