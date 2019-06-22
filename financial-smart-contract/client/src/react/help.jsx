import React from "react";

/**
 * The Help component.
 */
export default class Help extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "help";

    /**
     * Returns the element that represents this component.
     */
    render() {
        return (
            <div className={Help.blockName + "__container"}>
                <h2 className={Help.blockName + "__header"}>
                    SmartFin - The Financial Contract DSL
                </h2>

                <h3 className={Help.blockName + "__subheader"}>
                    Overview
                </h3>

                <span className={Help.blockName + "__text"}>
                    This help menu will describe how SmartFin contracts operate. 
                    For more general information about how to use the client, 
                    please refer to the user manual instead.
                    <br/><br/>

                    The financial smart contract client deals with smart 
                    contracts that represent financial contracts defined in 
                    SmartFin, a combinator language used to describe financial 
                    contracts. A combinator language is a functional language in 
                    which a program is made up of chained function calls. As 
                    such, each contract written in SmartFin can be used as a 
                    sub-contract for any combinator. This means that a whole 
                    inancial contract can be represented by a single combinator, 
                    or by some composition of combinators.
                    <br/><br/>

                    Each SmartFin financial contract has a <em>holder</em>, and 
                    a <em>counter-party</em>. Typically, the counter-party will 
                    be the party making payments, and the holder will be the 
                    party receiving payments. All payments are made in Ether 
                    (and all amounts of currency will be in the form of Wei, 
                    the smallest denomination of Ether).
                    <br/><br/>

                    A SmartFin financial contract can be <em>acquired</em> by 
                    the holder at any point in time, but the responsibilities of 
                    each party may differ depending on when the contract is 
                    acquired. For example, consider a contract <em>c</em> which 
                    requires the counter-party to pay the holder 100 Ether on 
                    noon of January 1st 2019 and again on noon of January 1st 
                    2020. <em>c</em> requires 2 payments to occur if acquired before 
                    12:00 on 01/01/19, 1 payment to occur if acquired by the 
                    12:00 01/01/20, or none otherwise. The acquisition date of a 
                    SmartFin financial contract will therefore affect the value 
                    of the contract for each party.
                    <br/><br/>

                    A SmartFin financial contract may also <em>expire</em> where 
                    no responsibilities outlined in the contract take effect if 
                    the contract is acquired after a certain time. For example, 
                    the contract <em>c</em> has no effect if acquired after 12:00 on 
                    01/01/20. This date is called the <em>horizon</em> of the 
                    SmartFin contract. An important thing to note is that a 
                    contract's responsibilities could potentially extend past 
                    the contract's horizon, but a contract acquired after its 
                    horizon will have no effect.
                    <br/><br/>

                    Some SmartFin financial contracts may be dependent on not 
                    just sub-contracts, but also parameters. The 
                    contract <em>c</em>, for instance, defines payments of a 
                    specific amount on two 
                    specific dates. This contract would need to be defined with 
                    a constant representing 100 Ether, and two date/times. A 
                    SmartFin financial contract could also be dependent on a 
                    variable value, such as the temperature in London in 
                    Celsius, or the distance between two people in metres. 
                    Such a value is called an <em>observable</em>.
                </span>

                <br/><br/>

                <h3 className={Help.blockName + "__subheader"}>
                    Combinators
                </h3>

                <span className={Help.blockName + "__text"}>
                    The set of combinators defined in SmartFin is described 
                    below, along with the type signature of each combinator 
                    (using the function signature notation of 
                    Haskell). <em>c</em> and <em>d</em> represent 
                    contracts, <em>o</em> represents an observable, 
                    and <em>t</em> represents a time.
                </span>
                
                <h4 className={Help.blockName + "__subsubheader"}>
                        zero :: Contract
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    This combinator represents a contract with no terms. It can 
                    be acquired at any time.
                </span>
                
                <h4 className={Help.blockName + "__subsubheader"}>
                    one :: Contract
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    This combinator represents a contract which requires the 
                    counter-party to immediately pay the holder one Wei upon 
                    acquisition. This contract can be acquired at any time.
                </span>
                
                <h4 className={Help.blockName + "__subsubheader"}>
                    give :: Contract -> Contract
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    <em>give c</em> represents <em>c</em> with all 
                    responsibilities reversed (e.g. if the holder acquires <em>give one</em>, 
                    they must pay the counter-party 1 Wei 
                    immediately).
                </span>
                
                <h4 className={Help.blockName + "__subsubheader"}>
                    and :: Contract -> Contract -> Contract
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    When <em>and c d</em> is acquired, both <em>c</em> and <em>d</em> are 
                    acquired immediately. Expired sub-contracts are not acquired.
                </span>
                
                <h4 className={Help.blockName + "__subsubheader"}>
                    or :: Contract -> Contract -> Contract
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    When <em>or c d</em> is acquired, the holder immediately 
                    acquires either c or d. If one has expired, the holder 
                    cannot acquire it (and must acquire the other if possible).
                </span>
                
                <h4 className={Help.blockName + "__subsubheader"}>
                    truncate :: Date -> Contract -> Contract
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    When <em>truncate t c</em> is acquired, the holder acquires <em>c</em>. 
                    The horizon of <em>truncate t c</em> is the earliest 
                    of <em>t</em> and the horizon of <em>c</em> (thus <em>truncate t c</em> does 
                    nothing after either horizon has passed).
                    <br/><br/>

                    Dates in SmartFin must be provided in either the 
                    format <em>&lt;DD/MM/YYYY HH:mm:ss&gt;</em>, the 
                    format <em>&gt;DD/MM/YYYY HH:mm:ss Z&gt;</em>, or in UNIX 
                    Epoch time format.
                </span>
                
                <h4 className={Help.blockName + "__subsubheader"}>
                    then :: Contract -> Contract -> Contract
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    When acquiring <em>then c d</em>, the holder 
                    acquires <em>c</em> if <em>c</em> has not expired, 
                    or <em>d</em> if <em>c</em> has expired and <em>d</em> has not.
                </span>
                
                <h4 className={Help.blockName + "__subsubheader"}>
                    scale :: Observable -> Contract -> Contract
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    <em>scale o c</em> represents <em>c</em> with all payments 
                    multiplied by the value of the observable <em>o</em> at 
                    the time of acquisition.
                    <br/><br/>

                    An observable is represented by either a number 
                    (e.g. <em>scale 5 one</em> requires the counter-party to pay 5 Wei 
                    to the holder), or by a name and address if the observable 
                    has a time-varying value. The name is used to refer to the 
                    observable in the financial smart contract client, and the 
                    address is the user address of an arbiter for the 
                    observable's value that will provide its value at some 
                    point. This is written in the 
                    form <em>scale &lt;name&gt; &lt;addr&gt; c</em>, 
                    e.g. <em>scale tempInLondon 
                        0xA0a4D3524dC3428884c41C05CD344f9BcB5c79f3 one</em>. 
                    Observable names can be in any form as long as they contain 
                    at least 1 non-mathematical character, such as a letter.
                </span>
                
                <h4 className={Help.blockName + "__subsubheader"}>
                    get :: Contract -> Contract
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    Acquiring <em>get c</em> acquires <em>c</em> at the moment 
                    in time when the horizon of <em>c</em> is reached. For 
                    example, <em>get truncate t one</em> will require the 
                    counter-party to pay the holder 1 Wei at time <em>t</em> (if 
                    acquired before it expires).
                </span>

                <h4 className={Help.blockName + "__subsubheader"}>
                    anytime :: Contract -> Contract
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    After <em>anytime c</em> is acquired, <em>c</em> can be 
                    acquired by the holder at any time before it expires, and 
                    must be acquired by this point.
                </span>

                <br/><br/>

                <h3 className={Help.blockName + "__subheader"}>
                    Examples
                </h3>

                <h4 className={Help.blockName + "__subsubheader"}>
                    Zero-Coupon Discount Bond
                </h4>

                <span className={Help.blockName + "__text"}>
                    One example of a simple financial contract is a 
                    zero-coupon discount bond. This is a contract between a 
                    holder and a counter-party that requires the counter-party 
                    to pay a specified amount of currency to the holder at a 
                    certain date.
                    <br/><br/>

                    A zero-coupon discount bond which requires the counter-party 
                    to pay 100 Wei to the holder at 12:00pm on 01/01/2020 is 
                    defined in SmartFin as:
                    <br/><br/>

                    <em>get truncate &lt;01/01/2020 12:00:00&gt; scale 100 one</em>
                    <br/><br/>

                    Once the <em>get</em> combinator is acquired, its 
                    sub-contract will be acquired at the acquisition date, 
                    i.e. 12:00pm on 01/01/01. The <em>truncate</em> combinator 
                    will not yet have expired, and so its underlying contract 
                    will be acquired at this point. The acquisition of 
                    the <em>scale</em> combinator causes its underlying contract 
                    (with values multiplied by 100) to be acquired immediately, 
                    thus acquiring the <em>one</em> combinator. This results in 
                    the counter-party paying 100 Wei to the holder at 12:00pm on 
                    01/01/2020, if acquired before this time.
                </span>


                <h4 className={Help.blockName + "__subsubheader"}>
                    European Option
                </h4>
                
                <span className={Help.blockName + "__text"}>
                    A European option is another type of financial contract, 
                    which states that the holder can choose whether or not to 
                    acquire a contract on a given date.
                    <br/><br/>

                    A European option over the contract <em>c</em> at 12:00pm 
                    on 01/01/2020 is defined in SmartFin as:
                    <br/><br/>

                    <em>get truncate &lt;01/01/2020 12:00:00&gt; or c zero</em>
                    <br/><br/>

                    Similarly to the previous contract, acquiring 
                    the <em>get</em> combinator acquires the sub-contract at its 
                    horizon, i.e. 12:00pm on 01/01/2020. This acquires the 
                    non-expired <em>truncate</em> combinator, and thus the 
                    underlying <em>or</em> combinator.

                    At any point in time, the holder may specify which branch of 
                    the <em>or</em> combinator they would like to acquire. In 
                    the financial smart contract implementation, the contract 
                    will not proceed until a choice is made. After a choice is 
                    made, the chosen branch is evaluated based on the 
                    acquisition time of the <em>or</em> combinator (i.e. 
                    01/01/2020 12:00pm), regardless of when the <em>or</em> choice 
                    is actually supplied.

                    This means that the user will select between the underlying 
                    contract <em>c</em> and <em>zero</em>, and the result will 
                    be paid out at 01/01/2020 12:00pm, or as soon as the <em>or</em> choice 
                    is provided (whichever is latest).
                </span>
            </div>
        )
    }
}