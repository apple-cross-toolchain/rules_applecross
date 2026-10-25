import SwiftUI

final class Box { var n = 0 }
func compute() -> Int { 1 }
extension EnvironmentValues {
    @Entry var arr: [Int] = [1]
    @Entry var mem: Color = Color.red
    @Entry var call: Int = compute()
    @Entry var noType = Box()
    @Entry var str: String = "s"
    @Entry var closure: () -> Void = {}
    @Entry var cast: Int = Int(3.0)
}
