//
// UKnn.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class UKnnInput : MLFeatureProvider {

    /// x as 2 element vector of floats
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
    public convenience init(x: MLShapedArray<Float>) {
        self.init(x: MLMultiArray(x))
    }

}


/// Model Prediction Output Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class UKnnOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// label as string value
    public var label: String {
        provider.featureValue(for: "label")!.stringValue
    }

    /// labelProbs as dictionary of strings to doubles
    public var labelProbs: [String : Double] {
        provider.featureValue(for: "labelProbs")!.dictionaryValue as! [String : Double]
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(label: String, labelProbs: [String : Double]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["label" : MLFeatureValue(string: label), "labelProbs" : MLFeatureValue(dictionary: labelProbs as [AnyHashable : NSNumber])])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Model Update Input Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class UKnnTrainingInput : MLFeatureProvider {

    /// x as 2 element vector of floats
    public var x: MLMultiArray

    /// label as string value
    public var label: String

    public var featureNames: Set<String> { ["x", "label"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "x" {
            return MLFeatureValue(multiArray: x)
        }
        if featureName == "label" {
            return MLFeatureValue(string: label)
        }
        return nil
    }

    public init(x: MLMultiArray, label: String) {
        self.x = x
        self.label = label
    }

    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public convenience init(x: MLShapedArray<Float>, label: String) {
        self.init(x: MLMultiArray(x), label: label)
    }

}


/// Class for model loading and prediction
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class UKnn {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "UKnn", withExtension:"mlmodelc")!
    }

    /**
        Construct UKnn instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of UKnn.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `UKnn.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct UKnn instance by automatically loading the model from the app's bundle.
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
        Construct UKnn instance with explicit path to mlmodelc file
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
        Construct UKnn instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<UKnn, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct UKnn instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> UKnn {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct UKnn instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<UKnn, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(UKnn(model: model)))
            }
        }
    }

    /**
        Construct UKnn instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> UKnn {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return UKnn(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as UKnnInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as UKnnOutput
    */
    public func prediction(input: UKnnInput) throws -> UKnnOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as UKnnInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as UKnnOutput
    */
    public func prediction(input: UKnnInput, options: MLPredictionOptions) throws -> UKnnOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return UKnnOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as UKnnInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as UKnnOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: UKnnInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> UKnnOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return UKnnOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 2 element vector of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as UKnnOutput
    */
    public func prediction(x: MLMultiArray) throws -> UKnnOutput {
        let input_ = UKnnInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 2 element vector of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as UKnnOutput
    */

    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public func prediction(x: MLShapedArray<Float>) throws -> UKnnOutput {
        let input_ = UKnnInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [UKnnInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [UKnnOutput]
    */
    public func predictions(inputs: [UKnnInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [UKnnOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [UKnnOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  UKnnOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
