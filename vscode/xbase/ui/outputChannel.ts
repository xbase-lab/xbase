import type vscode from "vscode";
import { window } from "vscode";
import { ContentLevel } from "../types";

export default class OutputChannel implements vscode.Disposable {
  private channel: vscode.OutputChannel;
  private shown = false;

  constructor() {
    this.channel = window.createOutputChannel("XBase", "xclog");
  }

  dispose() {
    this.channel.dispose();
  }

  /* show output */
  public show() {
    this.channel.show(true);
    this.channel.hide();
  }
  public toggle() {
    if (this.shown) {
      this.channel.hide();
      this.shown = false;
    } else {
      this.channel.show(true);
      this.shown = true;
    }
  }

  // TODO: output source code warnings & errors to Problems
  append(line: string, level: ContentLevel) {
    // TODO: find out based on vscode current log level
    this.channel.appendLine(line);
    switch (level) {
      case "Error": console.error(line); break;
      case "Warn": console.warn(line); break;
      case "Debug": console.debug(line); break;
      case "Info": console.info(line); break;
    }
  }
}
