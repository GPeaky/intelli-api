# This is the cap'n proto schema equivalent of the provided protobuf definition

@0xc4f0f471c58b83f8;

struct PacketMotionData {
    carMotionData @0 :List(CarMotionData);
}

struct CarMotionData {
    worldPositionX @0 :Float32;
    worldPositionY @1 :Float32;
    worldPositionZ @2 :Float32;
    worldVelocityX @3 :Float32;
    worldVelocityY @4 :Float32;
    worldVelocityZ @5 :Float32;
    worldForwardDirX @6 :Int16;
    worldForwardDirY @7 :Int16;
    worldForwardDirZ @8 :Int16;
    worldRightDirX @9 :Int16;
    worldRightDirY @10 :Int16;
    worldRightDirZ @11 :Int16;
    gForceLateral @12 :Float32;
    gForceLongitudinal @13 :Float32;
    gForceVertical @14 :Float32;
    yaw @15 :Float32;
    pitch @16 :Float32;
    roll @17 :Float32;
}
