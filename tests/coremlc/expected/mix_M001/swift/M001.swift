//
// M001.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class M001Input : MLFeatureProvider {

    /// features as color (kCVPixelFormatType_32BGRA) image buffer, 10 pixels wide by 6 pixels high
    public var features: CVPixelBuffer

    /// z9 as grayscale (kCVPixelFormatType_OneComponent8) image buffer, 6 pixels wide by 8 pixels high
    public var z9: CVPixelBuffer

    public var featureNames: Set<String> { ["features", "z9"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "features" {
            return MLFeatureValue(pixelBuffer: features)
        }
        if featureName == "z9" {
            return MLFeatureValue(pixelBuffer: z9)
        }
        return nil
    }

    public init(features: CVPixelBuffer, z9: CVPixelBuffer) {
        self.features = features
        self.z9 = z9
    }

    public convenience init(featuresWith features: CGImage, z9With z9: CGImage) throws {
        self.init(features: try MLFeatureValue(cgImage: features, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!, z9: try MLFeatureValue(cgImage: z9, pixelsWide: 6, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent8, options: nil).imageBufferValue!)
    }

    public convenience init(featuresAt features: URL, z9At z9: URL) throws {
        self.init(features: try MLFeatureValue(imageAt: features, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!, z9: try MLFeatureValue(imageAt: z9, pixelsWide: 6, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent8, options: nil).imageBufferValue!)
    }

    public func setFeatures(with features: CGImage) throws  {
        self.features = try MLFeatureValue(cgImage: features, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!
    }

    public func setZ9(with z9: CGImage) throws  {
        self.z9 = try MLFeatureValue(cgImage: z9, pixelsWide: 6, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent8, options: nil).imageBufferValue!
    }

    public func setFeatures(with features: URL) throws  {
        self.features = try MLFeatureValue(imageAt: features, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!
    }

    public func setZ9(with z9: URL) throws  {
        self.z9 = try MLFeatureValue(imageAt: z9, pixelsWide: 6, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent8, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class M001Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// mean_features as 1 by 3 matrix of floats
    public var mean_features: MLMultiArray {
        provider.featureValue(for: "mean_features")!.multiArrayValue!
    }

    /// mean_features as 1 by 3 matrix of floats
    public var mean_featuresShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(mean_features)
    }

    /// mean_z9 as 1 by 1 matrix of floats
    public var mean_z9: MLMultiArray {
        provider.featureValue(for: "mean_z9")!.multiArrayValue!
    }

    /// mean_z9 as 1 by 1 matrix of floats
    public var mean_z9ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(mean_z9)
    }

    /// s_image as 1 by 1 matrix of 16-bit floats
    public var s_image: MLMultiArray {
        provider.featureValue(for: "s_image")!.multiArrayValue!
    }

    /// s_image as 1 by 1 matrix of 16-bit floats
    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #endif
    public var s_imageShapedArray: MLShapedArray<Float16> {
        MLShapedArray<Float16>(s_image)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(mean_features: MLMultiArray, mean_z9: MLMultiArray, s_image: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["mean_features" : MLFeatureValue(multiArray: mean_features), "mean_z9" : MLFeatureValue(multiArray: mean_z9), "s_image" : MLFeatureValue(multiArray: s_image)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}

/// Model Prediction State Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public final class M001State {
    public enum Name: String, CaseIterable {
        case image = "image"
    }

    let handle: MLState

    init(handle: MLState) {
        self.handle = handle
    }

    public func withMultiArray<R>(for stateName: Name, _ body: (MLMultiArray) throws -> R) rethrows -> R {
        try handle.withMultiArray(for: stateName.rawValue, body)
    }
}

/// Class for model loading and prediction
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class M001 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "M001", withExtension:"mlmodelc")!
    }

    /**
        Construct M001 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of M001.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `M001.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct a model with configuration

        - parameters:
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(configuration: MLModelConfiguration = MLModelConfiguration()) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct M001 instance with explicit path to mlmodelc file
        - parameters:
           - modelURL: the file url of the model

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL) throws {
        try self.init(model: MLModel(contentsOf: modelURL))
    }

    /**
        Construct a model with URL of the .mlmodelc directory and configuration

        - parameters:
           - modelURL: the file url of the model
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {
        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))
    }

    /**
        Construct M001 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<M001, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct M001 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> M001 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct M001 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<M001, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(M001(model: model)))
            }
        }
    }

    /**
        Construct M001 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> M001 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return M001(model: model)
    }

    /**
        Make a new state.

        Core ML framework will allocate the state buffers declared in the model.

        The allocated state buffers are initialized to zeros. To initialize with different values, use `.withMultiArray(for:)` to get the mutable `MLMultiArray`-view to the state buffer.

        ```swift
        let state = model.makeState()
        state.withMultiArray(for: .image) { stateMultiArray in
            stateMultiArray[0] = 0.42
        }
        ```
    */
    public func makeState() -> M001State {
        M001State(handle: model.makeState())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as M001Input
           - state: the state that the prediction will use and update.

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as M001Output
    */
    public func prediction(input: M001Input, using state: M001State) throws -> M001Output {
        try prediction(input: input, using: state, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as M001Input
           - state: the state that the prediction will use and update.
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as M001Output
    */
    public func prediction(input: M001Input, using state: M001State, options: MLPredictionOptions) throws -> M001Output {
        let outFeatures = try model.prediction(from: input, using: state.handle, options: options)
        return M001Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as M001Input
           - state: the state that the prediction will use and update.
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as M001Output
    */
    public func prediction(input: M001Input, using state: M001State, options: MLPredictionOptions = MLPredictionOptions()) async throws -> M001Output {
        let outFeatures = try await model.prediction(from: input, using: state.handle, options: options)
        return M001Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - features: color (kCVPixelFormatType_32BGRA) image buffer, 10 pixels wide by 6 pixels high
            - z9: grayscale (kCVPixelFormatType_OneComponent8) image buffer, 6 pixels wide by 8 pixels high
            - state: the state that the prediction will use and update.

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as M001Output
    */
    public func prediction(features: CVPixelBuffer, z9: CVPixelBuffer, using state: M001State) throws -> M001Output {
        let input_ = M001Input(features: features, z9: z9)
        return try prediction(input: input_, using: state)
    }
}
