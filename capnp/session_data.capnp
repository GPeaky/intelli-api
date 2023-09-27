@0xad3123e0d8f82b3b;

struct MarshalZone {
    zoneStart @0 :Float32; # Fracción (0..1) de cómo va la vuelta cuando comienza la zona del marshal
    zoneFlag @1 :Int8;    # -1 = inválido/desconocido, 0 = ninguno, 1 = verde, 2 = azul, 3 = amarillo
}

struct WeatherForecastSample {
    sessionType @0 :UInt8;                   # 0 = desconocido, 1 = P1, 2 = P2, etc.
    timeOffset @1 :UInt8;                    # Tiempo en minutos para el que es el pronóstico
    weather @2 :UInt8;                       # Clima - 0 = claro, 1 = nublado ligero, etc.
    trackTemperature @3 :Int8;               # Temp. de la pista en grados Celsius
    trackTemperatureChange @4 :Int8;         # Cambio en temp. de la pista – 0 = subir, 1 = bajar, 2 = sin cambio
    airTemperature @5 :Int8;                 # Temp. del aire en grados Celsius
    airTemperatureChange @6 :Int8;           # Cambio en temp. del aire – 0 = subir, 1 = bajar, 2 = sin cambio
    rainPercentage @7 :UInt8;                # Porcentaje de lluvia (0-100)
}

struct PacketSessionData {
    weather @0 :UInt8;
    trackTemperature @1 :Int8;
    airTemperature @2 :Int8;
    totalLaps @3 :UInt8;
    trackLength @4 :UInt16;
    sessionType @5 :UInt8;
    trackId @6 :Int8;
    formula @7 :UInt8;
    sessionTimeLeft @8 :UInt16;
    sessionDuration @9 :UInt16;
    pitSpeedLimit @10 :UInt8;
    gamePaused @11 :UInt8;
    isSpectating @12 :UInt8;
    spectatorCarIndex @13 :UInt8;
    sliProNativeSupport @14 :UInt8;
    numMarshalZones @15 :UInt8;
    marshalZones @16 :List(MarshalZone);           # Lista de MarshalZone
    safetyCarStatus @17 :UInt8;
    networkGame @18 :UInt8;
    numWeatherForecastSamples @19 :UInt8;
    weatherForecastSamples @20 :List(WeatherForecastSample); # Lista de WeatherForecastSample
    forecastAccuracy @21 :UInt8;
    aiDifficulty @22 :UInt8;
    seasonLinkIdentifier @23 :UInt32;
    weekendLinkIdentifier @24 :UInt32;
    sessionLinkIdentifier @25 :UInt32;
    pitStopWindowIdealLap @26 :UInt32;
    pitStopWindowLatestLap @27 :UInt32;
    pitStopRejoinPosition @28 :UInt32;
    steeringAssist @29 :UInt8;
    brakingAssist @30 :UInt8;
    gearboxAssist @31 :UInt8;
    pitAssist @32 :UInt8;
    pitReleaseAssist @33 :UInt8;
    ersAssist @34 :UInt8;
    drsAssist @35 :UInt8;
    dynamicRacingLine @36 :UInt8;
    dynamicRacingLineType @37 :UInt8;
    gameMode @38 :UInt8;
    ruleSet @39 :UInt8;
    timeOfDay @40 :UInt8;
    sessionLength @41 :UInt8;
    speedUnitsLeadPlayer @42 :UInt8;
    temperatureUnitsLeadPlayer @43 :UInt8;
    speedUnitsSecondaryPlayer @44 :UInt8;
    temperatureUnitsSecondaryPlayer @45 :UInt8;
    numSafetyCarPeriods @46 :UInt8;
    numVirtualSafetyCarPeriods @47 :UInt8;
    numRedFlagPeriods @48 :UInt8;
}
