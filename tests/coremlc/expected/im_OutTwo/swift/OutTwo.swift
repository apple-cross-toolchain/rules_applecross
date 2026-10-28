//
// OutTwo.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class OutTwoInput : MLFeatureProvider {

    /// x as 1 × 3 × 4 × 4 4-dimensional array of floats
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
public class OutTwoOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// first as color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
    public var first: CVPixelBuffer {
        provider.featureValue(for: "first")!.imageBufferValue!
    }

    /// second as grayscale (kCVPixelFormatType_OneComponent8) image buffer, 4 pixels wide by 4 pixels high
    public var second: CVPixelBuffer {
        provider.featureValue(for: "second")!.imageBufferValue!
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(first: CVPixelBuffer, second: CVPixelBuffer) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["first" : MLFeatureValue(pixelBuffer: first), "second" : MLFeatureValue(pixelBuffer: second)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class OutTwo {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "OutTwo", withExtension:"mlmodelc")!
    }

    /**
        Construct OutTwo instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of OutTwo.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `OutTwo.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct OutTwo instance with explicit path to mlmodelc file
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
        Construct OutTwo instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<OutTwo, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct OutTwo instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> OutTwo {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct OutTwo instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<OutTwo, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(OutTwo(model: model)))
            }
        }
    }

    /**
        Construct OutTwo instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> OutTwo {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return OutTwo(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as OutTwoInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutTwoOutput
    */
    public func prediction(input: OutTwoInput) throws -> OutTwoOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as OutTwoInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutTwoOutput
    */
    public func prediction(input: OutTwoInput, options: MLPredictionOptions) throws -> OutTwoOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return OutTwoOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as OutTwoInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutTwoOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: OutTwoInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> OutTwoOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return OutTwoOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 3 × 4 × 4 4-dimensional array of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutTwoOutput
    */
    public func prediction(x: MLMultiArray) throws -> OutTwoOutput {
        let input_ = OutTwoInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 3 × 4 × 4 4-dimensional array of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as OutTwoOutput
    */

    public func prediction(x: MLShapedArray<Float>) throws -> OutTwoOutput {
        let input_ = OutTwoInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [OutTwoInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [OutTwoOutput]
    */
    public func predictions(inputs: [OutTwoInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [OutTwoOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [OutTwoOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  OutTwoOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
