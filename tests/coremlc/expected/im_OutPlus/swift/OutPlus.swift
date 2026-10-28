//
// OutPlus.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class OutPlusInput : MLFeatureProvider {

    /// x as 1 × 3 × 8 × 8 4-dimensional array of floats
    public var x: MLMultiArray

    public var featureNames: Set<String> { ["x"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "x" {
            return MLFeatureValue(multiArray: x)
        }
        return nil
    }

    public init(x: MLMultiArray) {
        self.x = x
    }

    public convenience init(x: MLShapedArray<Float>) {
        self.init(x: MLMultiArray(x))
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class OutPlusOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// image as color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high
    public var image: CVPixelBuffer {
        provider.featureValue(for: "image")!.imageBufferValue!
    }

    /// mean as 1 element vector of floats
    public var mean: MLMultiArray {
        provider.featureValue(for: "mean")!.multiArrayValue!
    }

    /// mean as 1 element vector of floats
    public var meanShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(mean)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(image: CVPixelBuffer, mean: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["image" : MLFeatureValue(pixelBuffer: image), "mean" : MLFeatureValue(multiArray: mean)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class OutPlus {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "OutPlus", withExtension:"mlmodelc")!
    }

    /**
        Construct OutPlus instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of OutPlus.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `OutPlus.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct OutPlus instance with explicit path to mlmodelc file
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
        Construct OutPlus instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<OutPlus, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct OutPlus instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> OutPlus {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct OutPlus instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<OutPlus, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(OutPlus(model: model)))
            }
        }
    }

    /**
        Construct OutPlus instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> OutPlus {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return OutPlus(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as OutPlusInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutPlusOutput
    */
    public func prediction(input: OutPlusInput) throws -> OutPlusOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as OutPlusInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutPlusOutput
    */
    public func prediction(input: OutPlusInput, options: MLPredictionOptions) throws -> OutPlusOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return OutPlusOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as OutPlusInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutPlusOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: OutPlusInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> OutPlusOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return OutPlusOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 3 × 8 × 8 4-dimensional array of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutPlusOutput
    */
    public func prediction(x: MLMultiArray) throws -> OutPlusOutput {
        let input_ = OutPlusInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 3 × 8 × 8 4-dimensional array of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutPlusOutput
    */

    public func prediction(x: MLShapedArray<Float>) throws -> OutPlusOutput {
        let input_ = OutPlusInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [OutPlusInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [OutPlusOutput]
    */
    public func predictions(inputs: [OutPlusInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [OutPlusOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [OutPlusOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  OutPlusOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
