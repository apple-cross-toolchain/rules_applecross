import UIKit

final class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        _ = application
        _ = launchOptions

        let window = UIWindow(frame: UIScreen.main.bounds)
        let viewController = UIViewController()
        viewController.view.backgroundColor = .systemBackground

        let label = UILabel()
        label.text = greetingText()
        label.textAlignment = .center
        label.translatesAutoresizingMaskIntoConstraints = false

        viewController.view.addSubview(label)
        NSLayoutConstraint.activate([
            label.centerXAnchor.constraint(equalTo: viewController.view.centerXAnchor),
            label.centerYAnchor.constraint(equalTo: viewController.view.centerYAnchor),
        ])

        window.rootViewController = viewController
        window.makeKeyAndVisible()
        self.window = window

        return true
    }
}

private func greetingText() -> String {
    guard let url = Bundle.main.url(forResource: "Greeting", withExtension: "txt"),
          let text = try? String(contentsOf: url, encoding: .utf8) else {
        return "Hello from Linux RBE"
    }
    return text.trimmingCharacters(in: .whitespacesAndNewlines)
}

UIApplicationMain(
    CommandLine.argc,
    CommandLine.unsafeArgv,
    nil,
    NSStringFromClass(AppDelegate.self)
)
