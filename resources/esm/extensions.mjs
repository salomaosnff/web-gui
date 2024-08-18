export class Extension {
  constructor({
    id,
    public_url,
    main_script
  }) {
    this.id = id;
    this.public_url = public_url;
    this.main_script = main_script;
    this.active = false;
    this.exports = {};
    this.subscriptions = new Set()
  }

  getAssetUrl(url) {
    return `${this.public_url}/${url}`;
  }

  async activate() {
    if (this.active) {
      return;
    }

    this.active = true;

    this.exports = await this.main_script.activate?.({
      subscriptions: this.subscriptions
    }) ?? {};
  }

  async deactivate() {
    if (!this.active) {
      return;
    }

    this.active = false;

    await this.main_script.deactivate?.();

    this.subscriptions.forEach(subscription => subscription.dispose());
    this.subscriptions.clear();

    this.exports = {};
  }

  static async from_JSON(json) {
    if (!json.main_script_url) {
      throw new Error('main_script_url is required');
    }

    const main_script = await import(json.main_script_url);

    return new Extension({
      id: json.id,
      public_url: json.public_url,
      main_script
    });
  }
}

const extension_queue = [];
let current_extension = null;

export async function activate(extension_json) {
  if (!extension_json) {
    return;
  }

  extension_queue.push(extension_json);

  while (extension_queue.length > 0) {
    await current_extension;
    const extension_json = extension_queue.shift();

    if (!extension_json?.main_script_url) {
      continue;
    }

    await (current_extension = new Promise(async (resolve, reject) => {
      try {
        const extension = await Extension.from_JSON(extension_json);

        await extension.activate?.();

        resolve()

      } catch (err) {
        reject(err);
      } finally {
        current_extension = null;
      }
    }))
  }
}