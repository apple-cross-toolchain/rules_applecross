//
// Nms.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
public class NmsInput : MLFeatureProvider {

    /// confidence as 10 by 2 matrix of doubles
    public var confidence: MLMultiArray

    /// coordinates as 10 by 4 matrix of doubles
    public var coordinates: MLMultiArray

    public var featureNames: Set<String> { ["confidence", "coordinates"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "confidence" {
            return MLFeatureValue(multiArray: confidence)
        }
        if featureName == "coordinates" {
            return MLFeatureValue(multiArray: coordinates)
        }
        return nil
    }

    public init(confidence: MLMultiArray, coordinates: MLMultiArray) {
        self.confidence = confidence
        self.coordinates = coordinates
    }

    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public convenience init(confidence: MLShapedArray<Double>, coordinates: MLShapedArray<Double>) {
        self.init(confidence: MLMultiArray(confidence), coordinates: MLMultiArray(coordinates))
    }

}


/// Model Prediction Output Type
@available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
public class NmsOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// c as 0 by 2 matrix of doubles
    public var c: MLMultiArray {
        provider.featureValue(for: "c")!.multiArrayValue!
    }

    /// c as 0 by 2 matrix of doubles
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public var cShapedArray: MLShapedArray<Double> {
        MLShapedArray<Double>(c)
    }

    /// b as 0 by 4 matrix of doubles
    public var b: MLMultiArray {
        provider.featureValue(for: "b")!.multiArrayValue!
    }

    /// b as 0 by 4 matrix of doubles
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public var bShapedArray: MLShapedArray<Double> {
        MLShapedArray<Double>(b)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(c: MLMultiArray, b: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["c" : MLFeatureValue(multiArray: c), "b" : MLFeatureValue(multiArray: b)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
public class Nms {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Nms", withExtension:"mlmodelc")!
    }

    /**
        Construct Nms instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Nms.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Nms.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct Nms instance by automatically loading the model from the app's bundle.
    */
    @available(*, deprecated, message: "Use init(configuration:) instead and handle errors appropriately.")
    public convenience init() {
        try! self.init(contentsOf: type(of:self).urlOfModelInThisBundle)
    }

    /**
        Construct a model with configuration

        - parameters:
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(configuration: MLModelConfiguration) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Nms instance with explicit path to mlmodelc file
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
        Construct Nms instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Nms, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Nms instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Nms {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Nms instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Nms, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Nms(model: model)))
            }
        }
    }

    /**
        Construct Nms instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Nms {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Nms(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as NmsInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as NmsOutput
    */
    public func prediction(input: NmsInput) throws -> NmsOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as NmsInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as NmsOutput
    */
    public func prediction(input: NmsInput, options: MLPredictionOptions) throws -> NmsOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return NmsOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as NmsInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as NmsOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: NmsInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> NmsOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return NmsOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - confidence: 10 by 2 matrix of doubles
            - coordinates: 10 by 4 matrix of doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as NmsOutput
    */
    public func prediction(confidence: MLMultiArray, coordinates: MLMultiArray) throws -> NmsOutput {
        let input_ = NmsInput(confidence: confidence, coordinates: coordinates)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - confidence: 10 by 2 matrix of doubles
            - coordinates: 10 by 4 matrix of doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as NmsOutput
    */

    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public func prediction(confidence: MLShapedArray<Double>, coordinates: MLShapedArray<Double>) throws -> NmsOutput {
        let input_ = NmsInput(confidence: confidence, coordinates: coordinates)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [NmsInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [NmsOutput]
    */
    public func predictions(inputs: [NmsInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [NmsOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [NmsOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  NmsOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
