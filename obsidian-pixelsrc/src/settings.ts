import { App, PluginSettingTab, Setting } from 'obsidian';
import type PixelsrcPlugin from './main';

export interface PixelsrcSettings {
  defaultScale: number;
  showWarnings: boolean;
  showTransparency: boolean;
  enableLivePreview: boolean;
}

export const DEFAULT_SETTINGS: PixelsrcSettings = {
  defaultScale: 4,
  showWarnings: false,
  showTransparency: true,
  enableLivePreview: true,
};

export class PixelsrcSettingTab extends PluginSettingTab {
  plugin: PixelsrcPlugin;

  constructor(app: App, plugin: PixelsrcPlugin) {
    super(app, plugin);
    this.plugin = plugin;
  }

  display(): void {
    const { containerEl } = this;
    containerEl.empty();

    containerEl.createEl('h2', { text: 'PixelSrc Settings' });

    new Setting(containerEl)
      .setName('Default Scale')
      .setDesc('Scale factor for rendered sprites (1-16)')
      .addSlider((slider) =>
        slider
          .setLimits(1, 16, 1)
          .setValue(this.plugin.settings.defaultScale)
          .setDynamicTooltip()
          .onChange(async (value) => {
            this.plugin.settings.defaultScale = value;
            await this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName('Show Warnings')
      .setDesc('Display rendering warnings below sprites')
      .addToggle((toggle) =>
        toggle
          .setValue(this.plugin.settings.showWarnings)
          .onChange(async (value) => {
            this.plugin.settings.showWarnings = value;
            await this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName('Transparency Background')
      .setDesc('Show checkered background for transparent pixels')
      .addToggle((toggle) =>
        toggle
          .setValue(this.plugin.settings.showTransparency)
          .onChange(async (value) => {
            this.plugin.settings.showTransparency = value;
            await this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName('Live Preview')
      .setDesc('Show sprite preview while editing (requires restart)')
      .addToggle((toggle) =>
        toggle
          .setValue(this.plugin.settings.enableLivePreview)
          .onChange(async (value) => {
            this.plugin.settings.enableLivePreview = value;
            await this.plugin.saveSettings();
          })
      );

    containerEl.createEl('h3', { text: 'Usage' });
    containerEl.createEl('p', {
      text: 'Create a code block with language "pixelsrc" or "pxl":',
    });
    containerEl.createEl('pre', {
      text: '```pixelsrc\n{"type":"sprite","name":"dot","palette":{"{x}":"#FF0000"},"grid":["{x}"]}\n```',
    });

    containerEl.createEl('h3', { text: 'Links' });
    const linkContainer = containerEl.createDiv();
    linkContainer.createEl('a', {
      text: 'PixelSrc Documentation',
      href: 'https://github.com/pixelsrc/pixelsrc',
    });
  }
}
