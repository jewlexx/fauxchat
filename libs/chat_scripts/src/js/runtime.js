((globalThis) => {
  const core = Deno[Deno.internal].core;

  function argsToMessage(...args) {
    return args
      .map((arg) => (typeof arg === "string" ? arg : JSON.stringify(arg)))
      .join(" ");
  }

  globalThis.console = {
    log: (...args) => {
      core.print(`[out]: ${argsToMessage(...args)}\n`, false);
    },
    error: (...args) => {
      core.print(`[err]: ${argsToMessage(...args)}\n`, true);
    },
  };

  globalThis.fauxchat = {
    /**
     *
     * @param {string} message
     * @param {number} count
     * @param {number} delay
     * @param {string} username
     */
    send: (message, count = 1, delay = 0, username = "random") => {
      core.ops.op_send(message, count, delay, username);
    },
  };
})(globalThis);
