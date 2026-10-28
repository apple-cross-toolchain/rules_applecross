//
// F000.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class F000Input : MLFeatureProvider {

    /// x as 1 × 5 × 5 × 3 4-dimensional array of 32-bit integers
    public var x: MLMultiArray

    /// tokens as color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 8 pixels high
    public var tokens: CVPixelBuffer

    public var featureNames: Set<String> { ["x", "tokens"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "x" {
            return MLFeatureValue(multiArray: x)
        }
        if featureName == "tokens" {
            return MLFeatureValue(pixelBuffer: tokens)
        }
        return nil
    }

    public init(x: MLMultiArray, tokens: CVPixelBuffer) {
        self.x = x
        self.tokens = tokens
    }

    public convenience init(x: MLShapedArray<Int32>, tokens: CVPixelBuffer) {
        self.init(x: MLMultiArray(x), tokens: tokens)
    }

    public convenience init(x: MLMultiArray, tokensWith tokens: CGImage) throws {
        self.init(x: x, tokens: try MLFeatureValue(cgImage: tokens, pixelsWide: 4, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!)
    }

    public convenience init(x: MLShapedArray<Int32>, tokensWith tokens: CGImage) throws {
        self.init(x: MLMultiArray(x), tokens: try MLFeatureValue(cgImage: tokens, pixelsWide: 4, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!)
    }

    public convenience init(x: MLMultiArray, tokensAt tokens: URL) throws {
        self.init(x: x, tokens: try MLFeatureValue(imageAt: tokens, pixelsWide: 4, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!)
    }

    public convenience init(x: MLShapedArray<Int32>, tokensAt tokens: URL) throws {
        self.init(x: MLMultiArray(x), tokens: try MLFeatureValue(imageAt: tokens, pixelsWide: 4, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!)
    }

    public func setTokens(with tokens: CGImage) throws  {
        self.tokens = try MLFeatureValue(cgImage: tokens, pixelsWide: 4, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!
    }

    public func setTokens(with tokens: URL) throws  {
        self.tokens = try MLFeatureValue(imageAt: tokens, pixelsWide: 4, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class F000Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// o_x as 1 × 5 × 5 × 3 4-dimensional array of 32-bit integers
    public var o_x: MLMultiArray {
        provider.featureValue(for: "o_x")!.multiArrayValue!
    }

    /// o_x as 1 × 5 × 5 × 3 4-dimensional array of 32-bit integers
    public var o_xShapedArray: MLShapedArray<Int32> {
        MLShapedArray<Int32>(o_x)
    }

    /// mean_tokens as 1 by 3 matrix of floats
    public var mean_tokens: MLMultiArray {
        provider.featureValue(for: "mean_tokens")!.multiArrayValue!
    }

    /// mean_tokens as 1 by 3 matrix of floats
    public var mean_tokensShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(mean_tokens)
    }

    /// s_image as 4 × 4 × 3 3-dimensional array of 16-bit floats
    public var s_image: MLMultiArray {
        provider.featureValue(for: "s_image")!.multiArrayValue!
    }

    /// s_image as 4 × 4 × 3 3-dimensional array of 16-bit floats
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

    public init(o_x: MLMultiArray, mean_tokens: MLMultiArray, s_image: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["o_x" : MLFeatureValue(multiArray: o_x), "mean_tokens" : MLFeatureValue(multiArray: mean_tokens), "s_image" : MLFeatureValue(multiArray: s_image)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}

/// Model Prediction State Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public final class F000State {
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
public class F000 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "F000", withExtension:"mlmodelc")!
    }

    /**
        Construct F000 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of F000.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `F000.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct F000 instance with explicit path to mlmodelc file
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
        Construct F000 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<F000, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct F000 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> F000 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct F000 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<F000, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(F000(model: model)))
            }
        }
    }

    /**
        Construct F000 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> F000 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return F000(model: model)
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
    public func makeState() -> F000State {
        F000State(handle: model.makeState())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as F000Input
           - state: the state that the prediction will use and update.

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F000Output
    */
    public func prediction(input: F000Input, using state: F000State) throws -> F000Output {
        try prediction(input: input, using: state, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as F000Input
           - state: the state that the prediction will use and update.
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F000Output
    */
    public func prediction(input: F000Input, using state: F000State, options: MLPredictionOptions) throws -> F000Output {
        let outFeatures = try model.prediction(from: input, using: state.handle, options: options)
        return F000Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as F000Input
           - state: the state that the prediction will use and update.
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F000Output
    */
    public func prediction(input: F000Input, using state: F000State, options: MLPredictionOptions = MLPredictionOptions()) async throws -> F000Output {
        let outFeatures = try await model.prediction(from: input, using: state.handle, options: options)
        return F000Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 5 × 5 × 3 4-dimensional array of 32-bit integers
            - tokens: color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 8 pixels high
            - state: the state that the prediction will use and update.

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F000Output
    */
    public func prediction(x: MLMultiArray, tokens: CVPixelBuffer, using state: F000State) throws -> F000Output {
        let input_ = F000Input(x: x, tokens: tokens)
        return try prediction(input: input_, using: state)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 5 × 5 × 3 4-dimensional array of 32-bit integers
            - tokens: color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 8 pixels high
            - state: the state that the prediction will use and update.

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F000Output
    */

    public func prediction(x: MLShapedArray<Int32>, tokens: CVPixelBuffer, using state: F000State) throws -> F000Output {
        let input_ = F000Input(x: x, tokens: tokens)
        return try prediction(input: input_, using: state)
    }
}
