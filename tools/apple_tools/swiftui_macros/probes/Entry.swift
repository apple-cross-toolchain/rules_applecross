import SwiftUI

final class Box { var n = 0 }
struct NotSendable { var f: () -> Void = {} }

extension EnvironmentValues {
    @Entry var box: Box = Box()
    @Entry var boxOpt: Box? = nil
    @Entry var ns: NotSendable = NotSendable()
    @Entry public var publicCount: Int = 1
}
extension Transaction { @Entry var flag: Bool = false }
extension FocusedValues { @Entry var focusedName: String? }
extension ContainerValues { @Entry var badge: Int = 0 }
