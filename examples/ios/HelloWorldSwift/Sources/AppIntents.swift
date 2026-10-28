import AppIntents

enum Greeting: String, AppEnum {
  case hello, goodbye

  static var typeDisplayRepresentation: TypeDisplayRepresentation = "Greeting"
  static var caseDisplayRepresentations: [Greeting: DisplayRepresentation] = [
    .hello: "Hello",
    .goodbye: DisplayRepresentation(title: "Goodbye", subtitle: "See you", image: .init(systemName: "hand.wave")),
  ]
}

struct GreetIntent: AppIntent {
  static var title: LocalizedStringResource = "Greet"
  static var description = IntentDescription("Greets someone.", categoryName: "Greetings")

  @Parameter(title: "Name", default: "World")
  var name: String

  @Parameter(title: "Greeting")
  var greeting: Greeting

  @Parameter(title: "Times", default: 1, inclusiveRange: (1, 5))
  var times: Int

  static var parameterSummary: some ParameterSummary {
    Summary("Say \(\.$greeting) to \(\.$name)") {
      \.$times
    }
  }

  func perform() async throws -> some IntentResult & ProvidesDialog {
    .result(dialog: "\(greeting.rawValue), \(name)!")
  }
}

struct CountIntent: AppIntent {
  static var title: LocalizedStringResource = "Count"
  static var openAppWhenRun: Bool = true

  func perform() async throws -> some IntentResult & ReturnsValue<Int> {
    .result(value: 3)
  }
}

struct HelloWorldShortcuts: AppShortcutsProvider {
  static var shortcutTileColor: ShortcutTileColor = .teal

  static var appShortcuts: [AppShortcut] {
    AppShortcut(
      intent: GreetIntent(),
      phrases: ["Greet with \(.applicationName)"],
      shortTitle: "Greet",
      systemImageName: "hand.wave"
    )
  }
}
