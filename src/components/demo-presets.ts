// Shared configuration for the NeuralType demo islands. The nastaliq
// demo appears in more than one post; both import this preset, so
// editing it here updates every instance at once.
export const nastaliqDemo = {
  font: '/demos/neuraltype/gulzar.ntf',
  // نور على نور as the default while the الله ligature finishes
  // training; flip back to بسم الله after the fine-tune lands.
  text: 'نور على نور',
  samples: 'نور على نور,بسم الله,بسم الله الرحمن الرحيم,سلام,قلم',
}
