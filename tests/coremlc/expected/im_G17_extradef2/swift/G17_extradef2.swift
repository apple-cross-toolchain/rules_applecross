//
// G17_extradef2.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class G17_extradef2Input : MLFeatureProvider {

    /// img as grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 8 pixels wide by 8 pixels high
    public var img: CVPixelBuffer

    /// t as 2 element vector of 16-bit floats
    public var t: MLMultiArray

    public var featureNames: Set<String> { ["img", "t"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "img" {
            return MLFeatureValue(pixelBuffer: img)
        }
        if featureName == "t" {
            return MLFeatureValue(multiArray: t)
        }
        return nil
    }

    public init(img: CVPixelBuffer, t: MLMultiArray) {
        self.img = img
        self.t = t
    }

    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public convenience init(img: CVPixelBuffer, t: MLShapedArray<Float16>) {
        self.init(img: img, t: MLMultiArray(t))
    }

    public convenience init(imgWith img: CGImage, t: MLMultiArray) throws {
        self.init(img: try MLFeatureValue(cgImage: img, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!, t: t)
    }

    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public convenience init(imgWith img: CGImage, t: MLShapedArray<Float16>) throws {
        self.init(img: try MLFeatureValue(cgImage: img, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!, t: MLMultiArray(t))
    }

    public convenience init(imgAt img: URL, t: MLMultiArray) throws {
        self.init(img: try MLFeatureValue(imageAt: img, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!, t: t)
    }

    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public convenience init(imgAt img: URL, t: MLShapedArray<Float16>) throws {
        self.init(img: try MLFeatureValue(imageAt: img, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!, t: MLMultiArray(t))
    }

    public func setImg(with img: CGImage) throws  {
        self.img = try MLFeatureValue(cgImage: img, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!
    }

    public func setImg(with img: URL) throws  {
        self.img = try MLFeatureValue(imageAt: img, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class G17_extradef2Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// reduce_mean_0 as 1 by 1 matrix of 16-bit floats
    public var reduce_mean_0: MLMultiArray {
        provider.featureValue(for: "reduce_mean_0")!.multiArrayValue!
    }

    /// reduce_mean_0 as 1 by 1 matrix of 16-bit floats
    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public var reduce_mean_0ShapedArray: MLShapedArray<Float16> {
        MLShapedArray<Float16>(reduce_mean_0)
    }

    /// relu_0 as 2 element vector of 16-bit floats
    public var relu_0: MLMultiArray {
        provider.featureValue(for: "relu_0")!.multiArrayValue!
    }

    /// relu_0 as 2 element vector of 16-bit floats
    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public var relu_0ShapedArray: MLShapedArray<Float16> {
        MLShapedArray<Float16>(relu_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(reduce_mean_0: MLMultiArray, relu_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["reduce_mean_0" : MLFeatureValue(multiArray: reduce_mean_0), "relu_0" : MLFeatureValue(multiArray: relu_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class G17_extradef2 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "G17_extradef2", withExtension:"mlmodelc")!
    }

    /**
        Construct G17_extradef2 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of G17_extradef2.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `G17_extradef2.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct G17_extradef2 instance with explicit path to mlmodelc file
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
        Construct G17_extradef2 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<G17_extradef2, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct G17_extradef2 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> G17_extradef2 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct G17_extradef2 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<G17_extradef2, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(G17_extradef2(model: model)))
            }
        }
    }

    /**
        Construct G17_extradef2 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> G17_extradef2 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return G17_extradef2(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as G17_extradef2Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as G17_extradef2Output
    */
    public func prediction(input: G17_extradef2Input) throws -> G17_extradef2Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as G17_extradef2Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as G17_extradef2Output
    */
    public func prediction(input: G17_extradef2Input, options: MLPredictionOptions) throws -> G17_extradef2Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return G17_extradef2Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as G17_extradef2Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as G17_extradef2Output
    */
    public func prediction(input: G17_extradef2Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> G17_extradef2Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return G17_extradef2Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - img: grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 8 pixels wide by 8 pixels high
            - t: 2 element vector of 16-bit floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as G17_extradef2Output
    */
    public func prediction(img: CVPixelBuffer, t: MLMultiArray) throws -> G17_extradef2Output {
        let input_ = G17_extradef2Input(img: img, t: t)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - img: grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 8 pixels wide by 8 pixels high
            - t: 2 element vector of 16-bit floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as G17_extradef2Output
    */

    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public func prediction(img: CVPixelBuffer, t: MLShapedArray<Float16>) throws -> G17_extradef2Output {
        let input_ = G17_extradef2Input(img: img, t: t)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [G17_extradef2Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [G17_extradef2Output]
    */
    public func predictions(inputs: [G17_extradef2Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [G17_extradef2Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [G17_extradef2Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  G17_extradef2Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
