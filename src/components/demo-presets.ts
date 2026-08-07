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
  // Classical Persian, for Attar: طلب (Seeking, first of the Seven
  // Valleys in the Conference of the Birds) and درد عشق (the pain
  // of love, his signature theme).
  // Note the model's vocabulary is the 31 Arabic letters of Gulzar
  // (no Persian-only letters), and short words render best: its
  // corpus is all words up to three letters plus a few real longer
  // ones. Rejected as too rough: سيمرغ as one word, راه عشق,
  // دل و جان, منطق الطير, هفت شهر عشق, فقر و فنا, سوز عشق (dropped
  // dot on ز), and the valleys معرفت, استغنا, توحيد, حيرت (looked
  // fine in a static render but broke in the live demo).
  samples: 'بسم الله,بسم الله الرحمن الرحيم,نور على نور,طلب,درد عشق,قلم ورق',
}
