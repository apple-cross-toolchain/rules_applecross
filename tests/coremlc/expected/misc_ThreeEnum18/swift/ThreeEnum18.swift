//
// ThreeEnum18.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class ThreeEnum18Input : MLFeatureProvider {

    /// a as 1 by 4 matrix of floats
    public var a: MLMultiArray

    /// b as 3 by 2 matrix of floats
    public var b: MLMultiArray

    /// c as color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
    public var c: CVPixelBuffer

    public var featureNames: Set<String> { ["a", "b", "c"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "a" {
            return MLFeatureValue(multiArray: a)
        }
        if featureName == "b" {
            return MLFeatureValue(multiArray: b)
        }
        if featureName == "c" {
            return MLFeatureValue(pixelBuffer: c)
        }
        return nil
    }

    public init(a: MLMultiArray, b: MLMultiArray, c: CVPixelBuffer) {
        self.a = a
        self.b = b
        self.c = c
    }

    public convenience init(a: MLShapedArray<Float>, b: MLShapedArray<Float>, c: CVPixelBuffer) {
        self.init(a: MLMultiArray(a), b: MLMultiArray(b), c: c)
    }

    public convenience init(a: MLMultiArray, b: MLMultiArray, cWith c: CGImage) throws {
        self.init(a: a, b: b, c: try MLFeatureValue(cgImage: c, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!)
    }

    public convenience init(a: MLShapedArray<Float>, b: MLShapedArray<Float>, cWith c: CGImage) throws {
        self.init(a: MLMultiArray(a), b: MLMultiArray(b), c: try MLFeatureValue(cgImage: c, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!)
    }

    public convenience init(a: MLMultiArray, b: MLMultiArray, cAt c: URL) throws {
        self.init(a: a, b: b, c: try MLFeatureValue(imageAt: c, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!)
    }

    public convenience init(a: MLShapedArray<Float>, b: MLShapedArray<Float>, cAt c: URL) throws {
        self.init(a: MLMultiArray(a), b: MLMultiArray(b), c: try MLFeatureValue(imageAt: c, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!)
    }

    public func setC(with c: CGImage) throws  {
        self.c = try MLFeatureValue(cgImage: c, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

    public func setC(with c: URL) throws  {
        self.c = try MLFeatureValue(imageAt: c, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class ThreeEnum18Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// relu_0 as multidimensional array of floats
    public var relu_0: MLMultiArray {
        provider.featureValue(for: "relu_0")!.multiArrayValue!
    }

    /// relu_0 as multidimensional array of floats
    public var relu_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(relu_0)
    }

    /// relu_1 as multidimensional array of floats
    public var relu_1: MLMultiArray {
        provider.featureValue(for: "relu_1")!.multiArrayValue!
    }

    /// relu_1 as multidimensional array of floats
    public var relu_1ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(relu_1)
    }

    /// reduce_mean_0 as 1 by 3 matrix of floats
    public var reduce_mean_0: MLMultiArray {
        provider.featureValue(for: "reduce_mean_0")!.multiArrayValue!
    }

    /// reduce_mean_0 as 1 by 3 matrix of floats
    public var reduce_mean_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(reduce_mean_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(relu_0: MLMultiArray, relu_1: MLMultiArray, reduce_mean_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["relu_0" : MLFeatureValue(multiArray: relu_0), "relu_1" : MLFeatureValue(multiArray: relu_1), "reduce_mean_0" : MLFeatureValue(multiArray: reduce_mean_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class ThreeEnum18 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "ThreeEnum18", withExtension:"mlmodelc")!
    }

    /**
        Construct ThreeEnum18 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of ThreeEnum18.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `ThreeEnum18.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct ThreeEnum18 instance with explicit path to mlmodelc file
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
        Construct ThreeEnum18 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ThreeEnum18, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct ThreeEnum18 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ThreeEnum18 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct ThreeEnum18 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ThreeEnum18, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(ThreeEnum18(model: model)))
            }
        }
    }

    /**
        Construct ThreeEnum18 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ThreeEnum18 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return ThreeEnum18(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ThreeEnum18Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ThreeEnum18Output
    */
    public func prediction(input: ThreeEnum18Input) throws -> ThreeEnum18Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ThreeEnum18Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ThreeEnum18Output
    */
    public func prediction(input: ThreeEnum18Input, options: MLPredictionOptions) throws -> ThreeEnum18Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return ThreeEnum18Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ThreeEnum18Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ThreeEnum18Output
    */
    public func prediction(input: ThreeEnum18Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> ThreeEnum18Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return ThreeEnum18Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: 1 by 4 matrix of floats
            - b: 3 by 2 matrix of floats
            - c: color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ThreeEnum18Output
    */
    public func prediction(a: MLMultiArray, b: MLMultiArray, c: CVPixelBuffer) throws -> ThreeEnum18Output {
        let input_ = ThreeEnum18Input(a: a, b: b, c: c)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: 1 by 4 matrix of floats
            - b: 3 by 2 matrix of floats
            - c: color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ThreeEnum18Output
    */

    public func prediction(a: MLShapedArray<Float>, b: MLShapedArray<Float>, c: CVPixelBuffer) throws -> ThreeEnum18Output {
        let input_ = ThreeEnum18Input(a: a, b: b, c: c)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [ThreeEnum18Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [ThreeEnum18Output]
    */
    public func predictions(inputs: [ThreeEnum18Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [ThreeEnum18Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [ThreeEnum18Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  ThreeEnum18Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
