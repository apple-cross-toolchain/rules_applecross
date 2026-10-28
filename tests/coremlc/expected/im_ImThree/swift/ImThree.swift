//
// ImThree.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImThreeInput : MLFeatureProvider {

    /// a as color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
    public var a: CVPixelBuffer

    /// b as 1 by 2 matrix of floats
    public var b: MLMultiArray

    /// c as grayscale (kCVPixelFormatType_OneComponent8) image buffer, 5 pixels wide by 5 pixels high
    public var c: CVPixelBuffer

    public var featureNames: Set<String> { ["a", "b", "c"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "a" {
            return MLFeatureValue(pixelBuffer: a)
        }
        if featureName == "b" {
            return MLFeatureValue(multiArray: b)
        }
        if featureName == "c" {
            return MLFeatureValue(pixelBuffer: c)
        }
        return nil
    }

    public init(a: CVPixelBuffer, b: MLMultiArray, c: CVPixelBuffer) {
        self.a = a
        self.b = b
        self.c = c
    }

    public convenience init(a: CVPixelBuffer, b: MLShapedArray<Float>, c: CVPixelBuffer) {
        self.init(a: a, b: MLMultiArray(b), c: c)
    }

    public convenience init(aWith a: CGImage, b: MLMultiArray, cWith c: CGImage) throws {
        self.init(a: try MLFeatureValue(cgImage: a, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!, b: b, c: try MLFeatureValue(cgImage: c, pixelsWide: 5, pixelsHigh: 5, pixelFormatType: kCVPixelFormatType_OneComponent8, options: nil).imageBufferValue!)
    }

    public convenience init(aWith a: CGImage, b: MLShapedArray<Float>, cWith c: CGImage) throws {
        self.init(a: try MLFeatureValue(cgImage: a, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!, b: MLMultiArray(b), c: try MLFeatureValue(cgImage: c, pixelsWide: 5, pixelsHigh: 5, pixelFormatType: kCVPixelFormatType_OneComponent8, options: nil).imageBufferValue!)
    }

    public convenience init(aAt a: URL, b: MLMultiArray, cAt c: URL) throws {
        self.init(a: try MLFeatureValue(imageAt: a, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!, b: b, c: try MLFeatureValue(imageAt: c, pixelsWide: 5, pixelsHigh: 5, pixelFormatType: kCVPixelFormatType_OneComponent8, options: nil).imageBufferValue!)
    }

    public convenience init(aAt a: URL, b: MLShapedArray<Float>, cAt c: URL) throws {
        self.init(a: try MLFeatureValue(imageAt: a, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!, b: MLMultiArray(b), c: try MLFeatureValue(imageAt: c, pixelsWide: 5, pixelsHigh: 5, pixelFormatType: kCVPixelFormatType_OneComponent8, options: nil).imageBufferValue!)
    }

    public func setA(with a: CGImage) throws  {
        self.a = try MLFeatureValue(cgImage: a, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!
    }

    public func setC(with c: CGImage) throws  {
        self.c = try MLFeatureValue(cgImage: c, pixelsWide: 5, pixelsHigh: 5, pixelFormatType: kCVPixelFormatType_OneComponent8, options: nil).imageBufferValue!
    }

    public func setA(with a: URL) throws  {
        self.a = try MLFeatureValue(imageAt: a, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!
    }

    public func setC(with c: URL) throws  {
        self.c = try MLFeatureValue(imageAt: c, pixelsWide: 5, pixelsHigh: 5, pixelFormatType: kCVPixelFormatType_OneComponent8, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImThreeOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// reshape_0 as 1 element vector of floats
    public var reshape_0: MLMultiArray {
        provider.featureValue(for: "reshape_0")!.multiArrayValue!
    }

    /// reshape_0 as 1 element vector of floats
    public var reshape_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(reshape_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(reshape_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["reshape_0" : MLFeatureValue(multiArray: reshape_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImThree {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "ImThree", withExtension:"mlmodelc")!
    }

    /**
        Construct ImThree instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of ImThree.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `ImThree.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct ImThree instance with explicit path to mlmodelc file
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
        Construct ImThree instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ImThree, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct ImThree instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ImThree {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct ImThree instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ImThree, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(ImThree(model: model)))
            }
        }
    }

    /**
        Construct ImThree instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ImThree {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return ImThree(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImThreeInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImThreeOutput
    */
    public func prediction(input: ImThreeInput) throws -> ImThreeOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImThreeInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImThreeOutput
    */
    public func prediction(input: ImThreeInput, options: MLPredictionOptions) throws -> ImThreeOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return ImThreeOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImThreeInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImThreeOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: ImThreeInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> ImThreeOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return ImThreeOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
            - b: 1 by 2 matrix of floats
            - c: grayscale (kCVPixelFormatType_OneComponent8) image buffer, 5 pixels wide by 5 pixels high

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImThreeOutput
    */
    public func prediction(a: CVPixelBuffer, b: MLMultiArray, c: CVPixelBuffer) throws -> ImThreeOutput {
        let input_ = ImThreeInput(a: a, b: b, c: c)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
            - b: 1 by 2 matrix of floats
            - c: grayscale (kCVPixelFormatType_OneComponent8) image buffer, 5 pixels wide by 5 pixels high

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImThreeOutput
    */

    public func prediction(a: CVPixelBuffer, b: MLShapedArray<Float>, c: CVPixelBuffer) throws -> ImThreeOutput {
        let input_ = ImThreeInput(a: a, b: b, c: c)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [ImThreeInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [ImThreeOutput]
    */
    public func predictions(inputs: [ImThreeInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [ImThreeOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [ImThreeOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  ImThreeOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
