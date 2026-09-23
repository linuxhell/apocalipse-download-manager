(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  else root.ApocalipseAILocalModel = api;
})(typeof globalThis !== "undefined" ? globalThis : this, function () {
  "use strict";

  // Lightweight, offline intent model. Add real-world expressions here and
  // protect every addition with a regression test before shipping it.
  const intents = [
    { name: "update_check", exact: [
      "tem atualizacao", "ha atualizacao", "existe atualizacao", "estou na ultima versao", "qual a ultima versao", "tem versao nova", "saiu versao nova", "tem uma nova versao", "quero atualizar", "posso atualizar", "tem update",
      "check for updates", "is there an update", "am i up to date", "latest version", "any updates", "any new version", "is there a new version", "do i have the latest version", "new release available",
      "有更新吗", "检查更新", "是最新版本吗", "最新版本", "有新版本吗", "有没有更新", "可以更新吗", "需要更新吗",
    ] },
    { name: "current_time", exact: [
      "que horas sao", "qual e a hora", "qual o horario", "me diga as horas", "que horas", "voce sabe que horas sao",
      "what time is it", "what is the time", "current time", "do you know the time", "tell me the time",
      "现在几点", "几点了", "现在时间", "现在是几点", "告诉我时间",
    ] },
    { name: "acknowledgement", exact: [
      "ok", "okay", "certo", "ta certo", "esta certo", "bacana", "legal", "beleza", "entendi", "combinado", "perfeito", "otimo", "tudo bem", "tranquilo", "show", "blz", "valeu entao", "fechou", "de boa", "suave",
      "all right", "alright", "got it", "understood", "sounds good", "great", "fine", "nice", "cool", "roger", "makes sense", "gotcha", "sure",
      "好", "好的", "可以", "行", "明白", "明白了", "知道了", "没问题", "了解", "懂了", "收到",
    ] },
    { name: "thanks", exact: [
      "obrigado", "obrigada", "muito obrigado", "muito obrigada", "valeu", "agradeco", "agradecido", "te agradeco", "grato",
      "thanks", "thank you", "thank you very much", "thx", "thanks a lot", "much appreciated", "appreciate it", "many thanks",
      "谢谢", "多谢", "感谢", "谢谢你", "非常感谢", "太感谢了",
    ] },
    { name: "goodbye", exact: [
      "tchau", "ate mais", "ate logo", "falou", "ate a proxima", "nos vemos", "flw", "vou nessa",
      "bye", "goodbye", "see you", "see you later", "see ya", "talk to you later", "catch you later", "gotta go",
      "再见", "拜拜", "回头见", "下次见", "先走了", "88",
    ] },
    { name: "wellbeing", exact: [
      "como voce esta", "como vai", "tudo bem com voce", "tudo bem", "e ai tudo bem", "como esta indo", "tudo certo por ai",
      "how are you", "how is it going", "how are you doing", "how's it going", "you good", "all good",
      "你好吗", "你怎么样", "你还好吗", "最近怎么样", "还好吗",
    ] },
    { name: "help", exact: [
      "me ajuda", "pode me ajudar", "preciso de ajuda", "ajuda", "socorro", "me ajude", "voce pode me ajudar", "preciso de uma ajuda",
      "help", "help me", "can you help me", "i need help", "i need some help", "could you help me", "please help",
      "请帮帮我", "帮帮我", "我需要帮助", "帮我一下", "能帮我吗", "求助",
    ] },
    { name: "capabilities", contains: [
      "o que voce faz", "o que voce sabe fazer", "o que voce consegue fazer", "como voce pode ajudar", "suas funcoes", "para que voce serve", "o que voce e capaz de fazer", "quais suas funcionalidades",
      "what can you do", "how can you help", "your capabilities", "what are you capable of", "what do you do", "what are your features",
      "你能做什么", "你可以做什么", "怎么帮助", "你有什么功能", "你能干什么", "你会做什么",
    ] },
  ];

  const clean = value => String(value || "").trim().replace(/[?!.。！？]+$/u, "").trim();
  function classify(normalizedInput) {
    const input = clean(normalizedInput);
    for (const intent of intents) {
      if (intent.exact?.includes(input)) return { intent: intent.name, confidence: 1 };
      if (intent.contains?.some(phrase => input.includes(phrase))) return { intent: intent.name, confidence: 0.95 };
    }
    return null;
  }

  return { classify, intents };
});
