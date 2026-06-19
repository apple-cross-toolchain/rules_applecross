import UIKit

public enum HelloUIKit {
    public static func makeLabelText() -> String {
        let label = UILabel(frame: .zero)
        label.text = "Hello from Swift UIKit"
        return label.text ?? ""
    }
}
