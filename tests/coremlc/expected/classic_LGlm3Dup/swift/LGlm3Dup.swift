//
// LGlm3Dup.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0, *)
public class LGlm3DupInput : MLFeatureProvider {

    /// x as 2 element vector of doubles
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

    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public convenience init(x: MLShapedArray<Double>) {
        self.init(x: MLMultiArray(x))
    }

}


/// Model Prediction Output Type
@available(macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0, *)
public class LGlm3DupOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// label as string value
    public var label: String {
        provider.featureValue(for: "label")!.stringValue
    }

    /// probs as dictionary of strings to doubles
    public var probs: [String : Double] {
        provider.featureValue(for: "probs")!.dictionaryValue as! [String : Double]
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(label: String, probs: [String : Double]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["label" : MLFeatureValue(string: label), "probs" : MLFeatureValue(dictionary: probs as [AnyHashable : NSNumber])])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0, *)
public class LGlm3Dup {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "LGlm3Dup", withExtension:"mlmodelc")!
    }

    /**
        Construct LGlm3Dup instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of LGlm3Dup.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `LGlm3Dup.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct LGlm3Dup instance by automatically loading the model from the app's bundle.
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
    @available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
    public convenience init(configuration: MLModelConfiguration) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct LGlm3Dup instance with explicit path to mlmodelc file
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
    @available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {
        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))
    }

    /**
        Construct LGlm3Dup instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<LGlm3Dup, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct LGlm3Dup instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> LGlm3Dup {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct LGlm3Dup instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<LGlm3Dup, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(LGlm3Dup(model: model)))
            }
        }
    }

    /**
        Construct LGlm3Dup instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> LGlm3Dup {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return LGlm3Dup(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as LGlm3DupInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as LGlm3DupOutput
    */
    public func prediction(input: LGlm3DupInput) throws -> LGlm3DupOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as LGlm3DupInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as LGlm3DupOutput
    */
    public func prediction(input: LGlm3DupInput, options: MLPredictionOptions) throws -> LGlm3DupOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return LGlm3DupOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as LGlm3DupInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as LGlm3DupOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: LGlm3DupInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> LGlm3DupOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return LGlm3DupOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 2 element vector of doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as LGlm3DupOutput
    */
    public func prediction(x: MLMultiArray) throws -> LGlm3DupOutput {
        let input_ = LGlm3DupInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 2 element vector of doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as LGlm3DupOutput
    */

    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public func prediction(x: MLShapedArray<Double>) throws -> LGlm3DupOutput {
        let input_ = LGlm3DupInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [LGlm3DupInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [LGlm3DupOutput]
    */
    @available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
    public func predictions(inputs: [LGlm3DupInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [LGlm3DupOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [LGlm3DupOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  LGlm3DupOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
