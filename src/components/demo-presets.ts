// Shared configuration for the NeuralType demo islands. The nastaliq
// demo appears in more than one post; both import this preset, so
// editing it here updates every instance at once.
export const nastaliqDemo = {
  font: '/demos/neuraltype/gulzar.ntf',
  // The engine and the demo both support a second, vector-output
  // model: set `vectorFont` to a neuraltype-vector-v1 .ntf and a
  // model picker appears. Left unset because the trained vector
  // model draws fragments and the file is 53 MB.
  text: 'بسم الله',
  samples: 'بسم الله,بسم الله الرحمن الرحيم,نور على نور,سلام,قلم',
}
