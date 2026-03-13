export const JOINTS = [
  'pelvis', 'torso', 'neck', 'head',
  'shoulderL', 'shoulderR',
  'upperArmL', 'upperArmR',
  'elbowL', 'elbowR',
  'handL', 'handR',
  'hipL', 'hipR',
  'kneeL', 'kneeR',
  'footL', 'footR',
];

export const REST_POSE = {
  pelvis:    { rotX: 0,    rotY: 0,    rotZ: 0 },
  torso:     { rotX: 0,    rotY: 0,    rotZ: 0 },
  neck:      { rotX: 0,    rotY: 0,    rotZ: 0 },
  head:      { rotX: 0,    rotY: 0,    rotZ: 0 },
  shoulderL: { rotX: 0,    rotY: 0,    rotZ:  0.05 },
  shoulderR: { rotX: 0,    rotY: 0,    rotZ: -0.05 },
  upperArmL: { rotX: 0,    rotY: 0,    rotZ: 0 },
  upperArmR: { rotX: 0,    rotY: 0,    rotZ: 0 },
  elbowL:    { rotX: 0.1,  rotY: 0,    rotZ: 0 },
  elbowR:    { rotX: 0.1,  rotY: 0,    rotZ: 0 },
  handL:     { rotX: 0,    rotY: 0,    rotZ: 0 },
  handR:     { rotX: 0,    rotY: 0,    rotZ: 0 },
  hipL:      { rotX: 0,    rotY: 0,    rotZ: 0 },
  hipR:      { rotX: 0,    rotY: 0,    rotZ: 0 },
  kneeL:     { rotX: 0.02, rotY: 0,    rotZ: 0 },
  kneeR:     { rotX: 0.02, rotY: 0,    rotZ: 0 },
  footL:     { rotX: 0,    rotY: 0,    rotZ: 0 },
  footR:     { rotX: 0,    rotY: 0,    rotZ: 0 },
};

export const ANGLE_LIMITS = {
  elbowL: { minX: 0, maxX: 2.6 },
  elbowR: { minX: 0, maxX: 2.6 },
  kneeL:  { minX: 0, maxX: 2.5 },
  kneeR:  { minX: 0, maxX: 2.5 },
};
